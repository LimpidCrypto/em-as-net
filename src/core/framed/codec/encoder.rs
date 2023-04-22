use core::fmt::Display;
use bytes::BytesMut;
use crate::core::framed::IoError;

pub trait Encoder<Item> {
    type Error: Display;

    fn encode(&mut self, data: Item, dst: &mut BytesMut) -> Result<(), Self::Error>;
}
