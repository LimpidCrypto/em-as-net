use core::fmt::Display;
use bytes::BytesMut;
use crate::core::framed::{Framed, IoError};
use crate::core::io::{AsyncRead, AsyncWrite};

pub trait Decoder {
    type Item;
    type Error: From<IoError> + Display;

    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error>;

    fn decode_eof(&mut self, buf: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        match self.decode(buf)? {
            Some(frame) => Ok(Some(frame)),
            None => {
                if buf.is_empty() {
                    Ok(None)
                } else {
                    Err(IoError::DecodeError.into())
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
