#![no_std]
#![no_main]

use aya_ebpf::{
    bindings::xdp_action,
    macros::{map, xdp},
    maps::HashMap,
    programs::XdpContext,
};
use aya_log_ebpf::info;
use core::mem;
use ebpf_loadbalancer_common::BackendPorts;

mod bindings;
use bindings::{ethhdr, iphdr, udphdr};

// UDP packet protocol number
const IPPROTO_UDP: u8 = 0x0011;
// IP packet protocol numver
const ETH_P_IP: u16 = 0x0800;
// ETH packer header offset
const ETH_HDR_LEN: usize = mem::size_of::<ethhdr>();
// IP packet header offset
const IP_HDR_LEN: usize = mem::size_of::<iphdr>();

// The kernel provides maps in BPF programs as a means for userspace programs to communicate with the underlying XDP program, and visa versa.
#[map(name = "BACKEND_PORTS")]
static mut BACKEND_PORTS: HashMap<u16, BackendPorts> =
    HashMap::<u16, BackendPorts>::with_max_entries(10, 0);

#[xdp]
pub fn ebpf_loadbalancer(ctx: XdpContext) -> u32 {
    match try_ebpf_loadbalancer(ctx) {
        Ok(ret) => ret,
        Err(_) => xdp_action::XDP_ABORTED,
    }
}

fn try_ebpf_loadbalancer(ctx: XdpContext) -> Result<u32, u32> {
    info!(&ctx, "received a packet");

    // try parse eth packet, if failed, skip its processing.
    let eth = ptr_at::<ethhdr>(&ctx, 0).ok_or(xdp_action::XDP_PASS)?;

    // If it is not a IP packet, skip its processing.
    if unsafe { u16::from_be((*eth).h_proto) } != ETH_P_IP {
        return Ok(xdp_action::XDP_PASS);
    }

    // try parse ip packet, if failed, skip its processing.
    let ip = ptr_at::<iphdr>(&ctx, ETH_HDR_LEN).ok_or(xdp_action::XDP_PASS)?;

    // If it is not a UDP packet, skip its processing.
    if unsafe { (*ip).protocol } != IPPROTO_UDP {
        return Ok(xdp_action::XDP_PASS);
    }

    info!(&ctx, "received a UDP packet");

    // try parse udp packet, if failed, skip its processing.
    let udp = ptr_at_mut::<udphdr>(&ctx, ETH_HDR_LEN + IP_HDR_LEN).ok_or(xdp_action::XDP_PASS)?;

    // since parse is successful, we can successfully obtain the standard udp dest port
    let destination_port = unsafe { u16::from_be((*udp).dest) };

    // load balancer by backend ports ebpf map
    let backends = match unsafe { BACKEND_PORTS.get(&destination_port) } {
        Some(backends) => {
            info!(&ctx, "hit the load balancer for port");
            backends
        }
        None => {
            info!(&ctx, "no backends found for this port, skip processing");
            return Ok(xdp_action::XDP_PASS);
        }
    };

    // Array access out of bounds.
    // This may occur when map insert fails.
    if backends.index > backends.ports.len() - 1 {
        return Ok(xdp_action::XDP_ABORTED);
    }

    // round robin load balancing: select the corresponding backend from the current index
    let new_destination_port = backends.ports[backends.index];
    // redirect udp dest to the backend port
    unsafe { (*udp).dest = u16::from_be(new_destination_port) };

    info!(
        &ctx,
        "redirected port {} to {}", destination_port, new_destination_port
    );

    // round robin load balancing,
    // next time when a udp packet hits, this will be routed to the next backend.
    let mut new_backends = BackendPorts {
        ports: backends.ports,
        index: backends.index + 1,
    };

    // when the backends end is reached or when there are less than four backends (the corresponding backend port is 0 because it is not filled).,
    // return to the original backend so that the load can continue to rotate among the available backends.
    if new_backends.index > new_backends.ports.len() - 1
        || new_backends.ports[new_backends.index] == 0
    {
        new_backends.index = 0;
    }

    // https://man7.org/linux/man-pages/man2/bpf.2.html
    // BPF_MAP_UPDATE_ELEM
    // BPF_ANY -> Create a new element or update an existing element.
    // update bpf map, so that the kernel can receive updates for round robin load balancing state.
    match unsafe { BACKEND_PORTS.insert(&destination_port, &new_backends, 0) } {
        Ok(_) => {
            info!(&ctx, "index updated for port {}", destination_port);
            Ok(xdp_action::XDP_PASS)
        }
        Err(err) => {
            info!(&ctx, "error inserting index update: {}", err);
            Ok(xdp_action::XDP_ABORTED)
        }
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe { core::hint::unreachable_unchecked() }
}

// helper function to access typed ptr in Xdpcontext memory
#[inline(always)]
fn ptr_at<T>(ctx: &XdpContext, offset: usize) -> Option<*const T> {
    let start = ctx.data();
    let end = ctx.data_end();
    let len = mem::size_of::<T>();

    if start + offset + len > end {
        return None;
    }

    Some((start + offset) as *const T)
}

#[inline(always)]
fn ptr_at_mut<T>(ctx: &XdpContext, offset: usize) -> Option<*mut T> {
    let ptr = ptr_at::<T>(ctx, offset)?;
    Some(ptr as *mut T)
}
