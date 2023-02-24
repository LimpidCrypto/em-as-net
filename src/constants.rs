pub const SIOCGIFMTU: libc::c_ulong = 0x8921;
pub const _SIOCGIFINDEX: libc::c_ulong = 0x8933;
pub const _ETH_P_ALL: libc::c_short = 0x0003;
pub const TUNSETIFF: libc::c_ulong = 0x400454CA;
pub const _IFF_TUN: libc::c_int = 0x0001;
pub const IFF_TAP: libc::c_int = 0x0002;
pub const IFF_NO_PI: libc::c_int = 0x1000;

pub const ETHERNET_HEADER_LEN: usize = 14;
pub const ETHERNET_MTU: usize = 1500;
pub const TCP_MAX_PACKET_SIZE: usize = 4096;
