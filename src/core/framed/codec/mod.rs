use bytes::{BufMut, BytesMut};

mod decoder;
mod encoder;

pub use decoder::Decoder;
pub use encoder::Encoder;

use crate::core::framed::IoError;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Default)]
pub struct Codec(());

impl Codec {
    pub fn new() -> Self {
        Self(())
    }
}

impl Encoder<&[u8]> for Codec {
    type Error = IoError;

    fn encode(&mut self, data: &[u8], buf: &mut BytesMut) -> Result<(), Self::Error> {
        buf.reserve(data.len());
        buf.put(data);
        Ok(())
    }
}

impl Decoder for Codec {
    type Item = BytesMut;
    type Error = IoError;

    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if !buf.is_empty() {
            let len = buf.len();
            Ok(Some(buf.split_to(len)))
        } else {
            Ok(None)
        }
    }
}
