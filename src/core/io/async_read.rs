use crate::Err;
use alloc::boxed::Box;
use anyhow::Result;
use core::fmt::{Debug, Display};
use core::ops::DerefMut;
use core::pin::Pin;
use core::task::{Context, Poll};

#[cfg(not(feature = "std"))]
use crate::io::ReadBuf;
#[cfg(feature = "std")]
use tokio::io::ReadBuf;

use crate::core::framed::IoError;

pub trait AsyncRead {
    type Error: Debug + Display;

    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<Result<(), Self::Error>>;
}

macro_rules! deref_async_read {
    () => {
        fn poll_read(
            mut self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            buf: &mut ReadBuf<'_>,
        ) -> Poll<Result<()>> {
            match Pin::new(&mut **self).poll_read(cx, buf) {
                Poll::Ready(result) => match result {
                    Ok(_) => Poll::Ready(Ok(())),
                    Err(_) => Poll::Ready(Err!(IoError::DecodeWhileReadError)),
                },
                Poll::Pending => Poll::Pending,
            }
        }
    };
}

impl<T: ?Sized + AsyncRead + Unpin> AsyncRead for Box<T> {
    type Error = anyhow::Error;

    deref_async_read!();
}

impl<T: ?Sized + AsyncRead + Unpin> AsyncRead for &mut T {
    type Error = anyhow::Error;

    deref_async_read!();
}

impl<P> AsyncRead for Pin<P>
where
    P: DerefMut + Unpin,
    P::Target: AsyncRead,
{
    type Error = anyhow::Error;

    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        match self.get_mut().as_mut().poll_read(cx, buf) {
            Poll::Ready(result) => match result {
                Ok(()) => Poll::Ready(Ok(())),
                Err(err) => Poll::Ready(Err!(err)),
            },
            Poll::Pending => Poll::Pending,
        }
    }
}
