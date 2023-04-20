use alloc::boxed::Box;
use core::fmt::Debug;
use core::ops::DerefMut;
use core::pin::Pin;
use core::task::{Context, Poll};
use crate::core::framed::FramedException;

#[cfg(feature = "std")]
use tokio::io::ReadBuf;
#[cfg(not(feature = "std"))]
use crate::core::io::ReadBuf;

pub trait AsyncRead {
    type Error: Debug;

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
        ) -> Poll<Result<(), Self::Error>> {
            match Pin::new(&mut **self).poll_read(cx, buf) {
                Poll::Ready(result) => {
                    match result {
                        Ok(_) => {
                            Poll::Ready(Ok(()))
                        }
                        Err(_) => {Poll::Ready(Err(FramedException::UnableToRead))}
                    }
                }
                Poll::Pending => {Poll::Pending}
            }
        }
    };
}

impl<T: ?Sized + AsyncRead + Unpin> AsyncRead for Box<T> {
    type Error = FramedException;

    deref_async_read!();
}

impl<T: ?Sized + AsyncRead + Unpin> AsyncRead for &mut T {
    type Error = FramedException;

    deref_async_read!();
}

impl<P> AsyncRead for Pin<P>
    where
        P: DerefMut + Unpin,
        P::Target: AsyncRead,
{
    type Error = FramedException;

    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        match self.get_mut().as_mut().poll_read(cx, buf) {
            Poll::Ready(result) => {
                match result {
                    Ok(_) => {
                        Poll::Ready(Ok(()))
                    }
                    Err(_) => {Poll::Ready(Err(FramedException::UnableToRead))}
                }
            }
            Poll::Pending => {Poll::Pending}
        }
    }
}
