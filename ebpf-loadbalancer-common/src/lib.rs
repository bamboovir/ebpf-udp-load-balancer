#![no_std]

// To allow the userspace program to inform the XDP program as to which backend ports traffic should be distributed to
// The structs that create for BPF maps will need to be memory aligned to the value of mem::align_of::() (commonly, 4), and have no padding.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct BackendPorts {
    pub ports: [u16; 4],
    pub index: usize,
}

#[cfg(feature = "user")]
unsafe impl aya::Pod for BackendPorts {}
