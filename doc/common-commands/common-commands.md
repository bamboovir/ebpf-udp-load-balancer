# common-commands

---

```bash
# list all local network interface
ip link
ip -o link show | awk -F': ' '{print $2}'
ls /sys/class/net
```

```bash
RUST_LOG=info cargo xtask run -- --iface ens3
RUST_LOG=info cargo xtask run -- --iface lo
```

```bash
aya-tool generate task_struct > /tmp/vmlinux.rs
cargo xtask codegen
```

```bash
sudo socat TCP-LISTEN:9999,fork EXEC:"Ping",reuseadd
echo 'Pong' | nc 127.0.0.1 9999
```

```bash
# enabling the dummy kernel module
sudo modprobe dummy
# create dummy network interface
sudo ip netns exec testns ip link add dummy type dummy
sudo ip netns exec testns ip link show dummy
# assign the CIDR to dummy interface
sudo ip netns exec testns ip addr add 192.168.1.1/24 dev dummy
sudo ip netns exec testns ip link set dummy up
# delete dummy network interface
sudo ip netns exec testns ip link delete dummy type dummy
```