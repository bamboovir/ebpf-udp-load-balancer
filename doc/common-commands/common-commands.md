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
