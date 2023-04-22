use bytes::BytesMut;
use core::fmt::Display;

pub trait Encoder<Item> {
    type Error: Display;

    fn encode(&mut self, data: Item, dst: &mut BytesMut) -> Result<(), Self::Error>;
}
