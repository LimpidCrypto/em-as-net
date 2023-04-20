use bytes::BytesMut;

pub trait Encoder<Item> {
    type Error;

    fn encode(&mut self, data: Item, dst: &mut BytesMut) -> Result<(), Self::Error>;
}
