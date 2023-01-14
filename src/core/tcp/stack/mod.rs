pub mod device;


pub struct TcpStack<'a, const MTU: usize, const BUF: usize> {
    stack: embassy_net::Stack<embassy_net_driver_channel::Device<'a, MTU>>
}

impl<'a, const MTU: usize, const BUF: usize> TcpStack<'a, MTU, BUF> {
    pub fn new(nic_mac_address: [u8; 6]) {
        
    }
}