pub mod stack;

use embassy_net::{tcp::TcpSocket, Stack, StackResources};
use embassy_net_driver_channel::{State, Device};
use rand::Rng;


pub struct Tcp<'a, const MTU: usize, const BUF: usize> {
    socket: TcpSocket<'a>,
}

// impl<'a, const MTU: usize, const BUF: usize> Tcp<'a, MTU, BUF> {
//     pub fn new(nic_mac_address: [u8; 6]) -> Self {
//         let stack = 
//         let mut rx_buffer = [0; 4096];
//         let mut tx_buffer = [0; 4096];
//         let socket = TcpSocket::new(&stack, &mut rx_buffer, &mut tx_buffer);

//         Self {
//             socket: socket,
//         }
//     }
// }
