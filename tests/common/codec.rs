use bytes::{BufMut, BytesMut};
use tokio_util::codec::{Decoder, Encoder};

pub struct Codec {}

impl Codec {
    pub fn new() -> Self {
        Codec {}
    }
}

impl Decoder for Codec {
    type Item = BytesMut;
    type Error = std::io::Error;

    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<BytesMut>, std::io::Error> {
        if !buf.is_empty() {
            let len = buf.len();
            Ok(Some(buf.split_to(len)))
        } else {
            Ok(None)
        }
    }
}

impl Encoder<&[u8]> for Codec {
    type Error = std::io::Error;

    fn encode(&mut self, data: &[u8], buf: &mut BytesMut) -> Result<(), std::io::Error> {
        buf.reserve(data.len());
        buf.put(data);
        Ok(())
    }
}
