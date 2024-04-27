# ebpf-udp-load-balancer

A simple local ebpf-udp-load-balancer experiment inspired by Kong Blog

## Usage

```bash
Usage: ebpf-loadbalancer [OPTIONS]

Options:
      --iface <IFACE>                    [default: lo]
      --source-port <SOURCE_PORT>        [default: 0]
      --upstream-ports <UPSTREAM_PORTS>  
  -h, --help                             Print help
  -V, --version                          Print version
```

## Motivation

### Why Choose UDP for eBPF experiment?

#### Statelessness:

UDP is inherently stateless; it does not require the establishment of a connection before data transfer occurs. This absence of connection state allows UDP to operate with minimal overhead, making it ideal for applications that require fast, efficient communication without the need for maintaining a session or connection state across packets. The stateless nature significantly simplifies the design of load balancers as there is no need to keep track of connection states, which can be resource-intensive.

#### Packet-based Communication:

Unlike TCP, which is stream-based and ensures data delivery and order, UDP operates on a packet-based model that does not guarantee delivery, order, or error-free communications. This model is particularly beneficial for real-time applications such as streaming media, online gaming, and voice over IP (VoIP), where speed and low latency are more critical than perfect delivery.

#### Simplified Load Balancing:

Load balancing with TCP can be complex due to the need to maintain and synchronize connection state across multiple servers. With UDP's stateless property, load balancing becomes more straightforward since each packet is independent and does not require knowledge of previous interactions. This simplification can lead to more efficient and scalable implementations, particularly when leveraging modern eBPF technology.

### Role of eBPF in UDP Load Balancing

Extended Berkeley Packet Filter (eBPF) provides a powerful platform to implement networking solutions at the kernel level, allowing for high performance and flexibility. By utilizing eBPF, this project aims to embed the load balancing logic directly into the Linux kernel, bypassing traditional user-space limitations and enhancing the performance of UDP packet routing and distribution across backend servers.

This approach not only capitalizes on the lightweight nature of UDP but also harnesses the advanced capabilities of eBPF to perform packet processing with minimal latency and overhead. The combination of UDP's protocol characteristics with eBPF's performance benefits presents a compelling solution for building a high-performance load balancer tailored for applications requiring quick response times and high reliability in packet handling.

## Set up development environment

OS: `Ubuntu 22.04 LTS`
Arch: `x86_64`

```bash
# Update packge cache
sudo apt-get update
# Install bcc packages
sudo apt-get install bpfcc-tools linux-headers-$(uname -r)
# Install rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
# Install util package
sudo apt install \
  build-essential \
  software-properties-common \
  pkg-config \
  libelf-dev \
  openssl \
  libssl-dev \
  wget \
  curl \
  git \
  zip \
  unzip \
  jq \
  fzf \
  ripgrep \
  htop \
  socat \
  zsh \
  rsync
```

Config rust dependencies

```bash
# Install the Rust stable and nightly toolchains
cargo install cargo-generate
# Install bpf-linker
cargo install bpf-linker
# Install scaffolding for aya project template
cargo install cargo-generate
# Install for C code bindings in Rust
# https://aya-rs.dev/book/aya/aya-tool/
cargo install bindgen-cli
cargo install --git https://github.com/aya-rs/aya -- aya-tool
```

Config Golang development environment

```bash
sudo apt-get install bison
bash < <(curl -s -S -L https://raw.githubusercontent.com/moovweb/gvm/master/binscripts/gvm-installer)
gvm install go1.4 -B
gvm use go1.4
export GOROOT_BOOTSTRAP=$GOROOT
gvm install go1.17.13
gvm use go1.17.13
export GOROOT_BOOTSTRAP=$GOROOT
gvm install go1.20
gvm use go1.20
gvm use go1.20 --default
```

Setting local testing network namespace.

```bash
# enabling ipv4 ip forwarding
sysctl -w net.ipv4.ip_forward=1
# create network namespace
sudo ip netns add testns
sudo ip netns list
# use local lo interface for testing
sudo ip netns exec testns ip link set lo up
# create shell in test network namespace
sudo ip netns exec testns sudo su $USER -
# delete test network namespace
sudo ip netns delete testns
```

## Build eBPF

```bash
cargo xtask build-ebpf
```

To perform a release build use the `--release` flag.
Change the target architecture with the `--target` flag.

## Build Userspace

```bash
cargo build
```

## Run

```bash
RUST_LOG=info cargo xtask run
```

## Project structure

- `ebpf-loadbalancer-ebpf`: the XDP eBPF code that will be loaded into the kernel
- `ebpf-loadbalancer`: the userspace program which will load and initialize the eBPF program
- `ebpf-loadbalancer-common`: shared code between the kernel and userspace code
- `xtask`: build and run tooling

## Demonstration

Build multi-udp-server

```bash
cd multi-udp-server
go build -o ../bin/multi-udp-server
```

Start ebpf udp rr load balancer

```bash
RUST_LOG=info cargo xtask run -- \
  --iface lo \
  --source-port 8000 \
  --upstream-ports 8080 \
  --upstream-ports 8081 \
  --upstream-ports 8082
```

Start multiple upstream udp server

```bash
# create shell in test network namespace
sudo ip netns exec testns sudo su $USER -
# start multiple udp server upstream
./bin/multi-udp-server -ports '8080,8081,8082'
```

ebpf_loadbalancer log

```log
[2024-04-27T10:52:29Z INFO  ebpf_loadbalancer] Waiting for Ctrl-C...
[2024-04-27T10:53:39Z INFO  ebpf_loadbalancer] received a packet
[2024-04-27T10:53:39Z INFO  ebpf_loadbalancer] received a UDP packet
[2024-04-27T10:53:39Z INFO  ebpf_loadbalancer] hit the load balancer for port
[2024-04-27T10:53:39Z INFO  ebpf_loadbalancer] redirected port 8000 to 8080
[2024-04-27T10:53:39Z INFO  ebpf_loadbalancer] index updated for port 8000
[2024-04-27T10:53:57Z INFO  ebpf_loadbalancer] received a packet
[2024-04-27T10:53:57Z INFO  ebpf_loadbalancer] received a UDP packet
[2024-04-27T10:53:57Z INFO  ebpf_loadbalancer] hit the load balancer for port
[2024-04-27T10:53:57Z INFO  ebpf_loadbalancer] redirected port 8000 to 8081
[2024-04-27T10:53:57Z INFO  ebpf_loadbalancer] index updated for port 8000
[2024-04-27T10:54:03Z INFO  ebpf_loadbalancer] received a packet
[2024-04-27T10:54:03Z INFO  ebpf_loadbalancer] received a UDP packet
[2024-04-27T10:54:03Z INFO  ebpf_loadbalancer] hit the load balancer for port
[2024-04-27T10:54:03Z INFO  ebpf_loadbalancer] redirected port 8000 to 8082
[2024-04-27T10:54:03Z INFO  ebpf_loadbalancer] index updated for port 8000
[2024-04-27T10:54:15Z INFO  ebpf_loadbalancer] received a packet
[2024-04-27T10:54:15Z INFO  ebpf_loadbalancer] received a UDP packet
[2024-04-27T10:54:15Z INFO  ebpf_loadbalancer] hit the load balancer for port
[2024-04-27T10:54:15Z INFO  ebpf_loadbalancer] redirected port 8000 to 8080
[2024-04-27T10:54:15Z INFO  ebpf_loadbalancer] index updated for port 8000
[2024-04-27T10:54:19Z INFO  ebpf_loadbalancer] received a packet
[2024-04-27T10:54:19Z INFO  ebpf_loadbalancer] received a UDP packet
[2024-04-27T10:54:19Z INFO  ebpf_loadbalancer] hit the load balancer for port
[2024-04-27T10:54:19Z INFO  ebpf_loadbalancer] redirected port 8000 to 8081
[2024-04-27T10:54:19Z INFO  ebpf_loadbalancer] index updated for port 8000
[2024-04-27T10:54:21Z INFO  ebpf_loadbalancer] received a packet
[2024-04-27T10:54:21Z INFO  ebpf_loadbalancer] received a UDP packet
[2024-04-27T10:54:21Z INFO  ebpf_loadbalancer] hit the load balancer for port
[2024-04-27T10:54:21Z INFO  ebpf_loadbalancer] redirected port 8000 to 8082
[2024-04-27T10:54:21Z INFO  ebpf_loadbalancer] index updated for port 8000
[2024-04-27T10:54:22Z INFO  ebpf_loadbalancer] received a packet
[2024-04-27T10:54:22Z INFO  ebpf_loadbalancer] received a UDP packet
[2024-04-27T10:54:22Z INFO  ebpf_loadbalancer] hit the load balancer for port
[2024-04-27T10:54:22Z INFO  ebpf_loadbalancer] redirected port 8000 to 8080
[2024-04-27T10:54:22Z INFO  ebpf_loadbalancer] index updated for port 8000
[2024-04-27T10:54:23Z INFO  ebpf_loadbalancer] received a packet
[2024-04-27T10:54:23Z INFO  ebpf_loadbalancer] received a UDP packet
[2024-04-27T10:54:23Z INFO  ebpf_loadbalancer] hit the load balancer for port
[2024-04-27T10:54:23Z INFO  ebpf_loadbalancer] redirected port 8000 to 8081
[2024-04-27T10:54:23Z INFO  ebpf_loadbalancer] index updated for port 8000
```

multi-udp-server log

```log
UDP Server: 2024/04/27 03:50:52 Listening on 127.0.0.1:8080
UDP Server: 2024/04/27 03:50:52 Listening on 127.0.0.1:8081
UDP Server: 2024/04/27 03:50:52 Listening on 127.0.0.1:8082
UDP Server: 2024/04/27 03:53:39 Port 8080: 5 bytes received from 127.0.0.1:35636
UDP Server: 2024/04/27 03:53:39 Port 8080: buffer contents: ping
UDP Server: 2024/04/27 03:53:57 Port 8081: 5 bytes received from 127.0.0.1:57849
UDP Server: 2024/04/27 03:53:57 Port 8081: buffer contents: ping
UDP Server: 2024/04/27 03:54:03 Port 8082: 5 bytes received from 127.0.0.1:43998
UDP Server: 2024/04/27 03:54:03 Port 8082: buffer contents: ping
UDP Server: 2024/04/27 03:54:15 Port 8080: 5 bytes received from 127.0.0.1:54323
UDP Server: 2024/04/27 03:54:15 Port 8080: buffer contents: ping
UDP Server: 2024/04/27 03:54:19 Port 8081: 5 bytes received from 127.0.0.1:44524
UDP Server: 2024/04/27 03:54:19 Port 8081: buffer contents: ping
UDP Server: 2024/04/27 03:54:21 Port 8082: 5 bytes received from 127.0.0.1:39487
UDP Server: 2024/04/27 03:54:21 Port 8082: buffer contents: ping
UDP Server: 2024/04/27 03:54:22 Port 8080: 5 bytes received from 127.0.0.1:46422
UDP Server: 2024/04/27 03:54:22 Port 8080: buffer contents: ping
UDP Server: 2024/04/27 03:54:23 Port 8081: 5 bytes received from 127.0.0.1:51190
UDP Server: 2024/04/27 03:54:23 Port 8081: buffer contents: ping
```

## Resource Link

- [Aya](https://github.com/aya-rs/aya)
- [Aya book](https://aya-rs.dev/book/)
- [Aya development environment](https://aya-rs.dev/book/start/development/)
- [Linux linux-interfaces](https://developers.redhat.com/blog/2018/10/22/introduction-to-linux-interfaces-for-virtual-networking)
- [bpf(2) — Linux manual page](https://man7.org/linux/man-pages/man2/bpf.2.html)
- [Writing an eBPF/XDP load-balancer in Rust](https://konghq.com/blog/engineering/writing-an-ebpf-xdp-load-balancer-in-rust)