// use alloc::borrow::Cow;
// use alloc::boxed::Box;
// use core::cell::RefCell;
// use core::future::Future;
// use core::pin::Pin;
// use core::task::{Context, Poll};
// use embedded_tls::{Aes128GcmSha256, NoVerify, TlsCipherSuite, TlsConfig, TlsContext, TlsError};
// use embedded_io::asynch::{Read, Write};
// use futures::{AsyncRead, FutureExt};
// use rand_core::OsRng;
// use tokio::io::ReadBuf;
// use crate::core::framed::FramedException;
// use crate::core::io;
// use crate::core::tcp::{Connect, TcpException};
// use super::TcpStream;
//
// pub struct TlsConnection<'a, S, C>
// where
//     S: Read + Write + 'a,
//     C: TlsCipherSuite + 'static,
// {
//     inner: RefCell<Option<embedded_tls::TlsConnection<'a, S, C>>>,
// }
//
// impl<'a, S, C> TlsConnection<'a, S, C>
// where
//     S: Read + Write + 'a,
//     C: TlsCipherSuite + 'static,
// {
//     pub fn new(delegate: S) -> Self {
//         let mut read_record_buffer = [0; 16384];
//         let mut write_record_buffer = [0; 16384];
//         // let mut rng = OsRng;
//         // let config = TlsConfig::new().with_server_name(&*server_name);
//         let tls: embedded_tls::TlsConnection<S, C> =
//             embedded_tls::TlsConnection::new(delegate, &mut read_record_buffer, &mut write_record_buffer);
//
//         Self {
//             inner: RefCell::new(Some(tls))
//         }
//     }
// }
//
// // impl<'a, S, C> Connect<'a> for TlsConnection<'a, S, C>
// // where
// //     S: Read + Write + 'a,
// //     C: TlsCipherSuite + 'static,
// // {
// //     type Error = TcpException;
// //
// //     async fn connect(&self, ip: &Cow<'a, str>) -> Result<(), TcpException> {
// //         let stream = TcpStream::new();
// //         stream.connect(ip).await.unwrap();
// //
// //         let mut read_record_buffer = [0; 16384];
// //         let mut write_record_buffer = [0; 16384];
// //
// //         Ok(())
// //    }
// // }
//
// impl<'a, S, C> io::AsyncRead for TlsConnection<'a, S, C>
// where
//     S: Read + Write + 'a,
//     C: TlsCipherSuite + 'static,
// {
//     type Error = FramedException;
//
//     fn poll_read(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut ReadBuf<'_>) -> Poll<Result<(), Self::Error>> {
//         // match self.inner.borrow_mut().as_mut() {
//         //     None => {Poll::Ready(Err(FramedException::UnableToRead))}
//         //     Some(stream) => {
//         //         match Box::pin(stream).read(buf.initialized_mut()).poll(cx) {
//         //             Poll::Ready(result) => {
//         //                 match result {
//         //                     Ok(_) => {Poll::Ready(Ok(()))}
//         //                     Err(_) => {Poll::Ready(Err(FramedException::UnableToRead))}
//         //                 }
//         //             },
//         //             Poll::Pending => {return Poll::Pending;}
//         //         }
//         //     }
//         // }
//
//         todo!()
//     }
// }
//
// impl<'a, S, C> io::AsyncWrite for TlsConnection<'a, S, C>
//     where
//         S: Read + Write + 'a,
//         C: TlsCipherSuite + 'static,
// {
//     fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<Result<usize, FramedException>> {
//         todo!()
//     }
//
//     fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), FramedException>> {
//         todo!()
//     }
//
//     fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), FramedException>> {
//         todo!()
//     }
// }
