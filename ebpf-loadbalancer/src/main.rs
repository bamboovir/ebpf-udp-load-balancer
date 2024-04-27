use anyhow::Context;
use aya::maps::HashMap;
use aya::programs::{Xdp, XdpFlags};
use aya::{include_bytes_aligned, Bpf};
use aya_log::BpfLogger;
use clap::Parser;
use ebpf_loadbalancer_common::BackendPorts;
use log::{debug, info, warn};
use tokio::signal;

#[derive(Debug, Parser)]
#[command(
    name = "ebpf-udp-load-balancer",
    version = "0.1.0",
    about = "A simple local ebpf-udp-load-balancer experiment inspired by Kong Blog"
)]
struct Opt {
    #[arg(long, default_value = "lo")]
    iface: String,
    #[arg(long, default_value_t = 0, value_parser = clap::value_parser!(u16))]
    source_port: u16,
    #[arg(long, value_parser = clap::value_parser!(u16))]
    upstream_ports: Vec<u16>,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let opt = Opt::parse();

    env_logger::init();

    // Bump the memlock rlimit. This is needed for older kernels that don't use the
    // new memcg based accounting, see https://lwn.net/Articles/837122/
    let rlim = libc::rlimit {
        rlim_cur: libc::RLIM_INFINITY,
        rlim_max: libc::RLIM_INFINITY,
    };
    let ret = unsafe { libc::setrlimit(libc::RLIMIT_MEMLOCK, &rlim) };
    if ret != 0 {
        debug!("remove limit on locked memory failed, ret is: {}", ret);
    }

    // This will include your eBPF object file as raw bytes at compile-time and load it at
    // runtime. This approach is recommended for most real-world use cases. If you would
    // like to specify the eBPF program at runtime rather than at compile-time, you can
    // reach for `Bpf::load_file` instead.
    #[cfg(debug_assertions)]
    let mut bpf = Bpf::load(include_bytes_aligned!(
        "../../target/bpfel-unknown-none/debug/ebpf-loadbalancer"
    ))?;
    #[cfg(not(debug_assertions))]
    let mut bpf = Bpf::load(include_bytes_aligned!(
        "../../target/bpfel-unknown-none/release/ebpf-loadbalancer"
    ))?;
    if let Err(e) = BpfLogger::init(&mut bpf) {
        // This can happen if you remove all log statements from your eBPF program.
        warn!("failed to initialize eBPF logger: {}", e);
    }
    let program: &mut Xdp = bpf.program_mut("ebpf_loadbalancer").unwrap().try_into()?;
    program.load()?;
    program.attach(&opt.iface, XdpFlags::default())
        .context("failed to attach the XDP program with default flags - try changing XdpFlags::default() to XdpFlags::SKB_MODE")?;

    // Initialize a map when loaded into the Kernel.
    // This map will associate the inbound destination port (as the map key)
    // with the Backends for that port (as the map value).
    // round robin load balancing:
    // Distribute client requests across a group of servers.
    // A client request is forwarded to each server in turn.
    // The algorithm instructs the load balancer to go back to the top of the list and repeats again.
    // port A -> [port B, port C, port D]
    // hit 0: port B
    // hit 1: port C
    // hit 2: port D
    // hit 3: port B
    // ...
    let mut backends: HashMap<_, u16, BackendPorts> = HashMap::try_from(
        bpf.map_mut("BACKEND_PORTS")
            .context("Failed to get BACKEND_PORTS")?,
    )?;

    let mut ports: [u16; 4] = [0; 4];

    for (i, &port) in opt.upstream_ports.iter().enumerate() {
        if i < 4 {
            ports[i] = port;
        } else {
            info!("Warning: More than four upstream ports were provided, only the first four upstream ports will be used for load balancing");
        }
    }

    // round robin load balancing start from first index
    let backend_ports = BackendPorts { ports, index: 0 };

    if opt.source_port != 0 {
        // https://man7.org/linux/man-pages/man2/bpf.2.html
        // BPF_MAP_UPDATE_ELEM
        // BPF_ANY -> Create a new element or update an existing element.
        backends.insert(opt.source_port, backend_ports, 0)?;
    }

    info!("Waiting for Ctrl-C...");
    signal::ctrl_c().await?;
    info!("Exiting...");

    Ok(())
}
