use embassy_net_driver_channel::State;


pub struct Device<'a, const MTU: usize, const BUF: usize> {
    // state: embassy_net_driver_channel::State<MTU, BUF, BUF>,
    device: embassy_net_driver_channel::Device<'a, MTU>
}

impl<'a, const MTU: usize, const BUF: usize> Device<'a, MTU, BUF> {
    pub fn new(nic_mac_address: [u8; 6]) -> Self {
        let mut state: State<MTU, BUF, BUF> = embassy_net_driver_channel::State::new();
        let (_, device) = embassy_net_driver_channel::new(
            &mut state,
            nic_mac_address,
        );

        Self {
            device
        }
    }
}
