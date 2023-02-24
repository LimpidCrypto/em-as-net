use embassy_net_driver_channel::State;

pub struct Nic<const MTU: usize, const MTU_COUNT: usize> {
    pub inner: embassy_net_driver_channel::Device<'static, MTU>,
}

impl<const MTU: usize, const MTU_COUNT: usize> Nic<MTU, MTU_COUNT> {
    pub fn new(
        nic_mac_address: [u8; 6],
        state: &'static mut embassy_net_driver_channel::State<MTU, MTU_COUNT, MTU_COUNT>,
    ) -> Self {
        let (_, device) = embassy_net_driver_channel::new(state, nic_mac_address);

        Self { inner: device }
    }
}
