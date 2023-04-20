use bytes::BytesMut;
use crate::core::framed::{Framed, FramedException};
use super::super::super::io::{AsyncRead, AsyncWrite};

pub trait Decoder {
    type Item;
    type Error: From<FramedException>;

    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error>;

    fn decode_eof(&mut self, buf: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        match self.decode(buf)? {
            Some(frame) => Ok(Some(frame)),
            None => {
                if buf.is_empty() {
                    Ok(None)
                } else {
                    Err(FramedException::DecodeError.into())
                }
            }
        }
    }

    fn framed<T: AsyncRead + AsyncWrite + Sized>(self, io: T) -> Framed<T, Self>
        where
            Self: Sized,
    {
        Framed::new(io, self)
    }
}
