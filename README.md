# em-as-net
embedded, async networking, utilizing the [`embassy`](https://github.com/embassy-rs/embassy) crate for `no_std` networking.
With `std` feature enabled we use [`tokio`](https://github.com/tokio-rs/tokio) crate for networking.

This crate is work in progress, currently focusing on the `std` part.

## core
### dns
Adapters for `no_std` DNS.
### framed
Some `no_std` implementation of [`tokio_util::codec`](https://github.com/tokio-rs/tokio/blob/master/tokio-util/src/codec):
- [`Encoder`](https://github.com/tokio-rs/tokio/blob/master/tokio-util/src/codec/encoder.rs)
- [`Decoder`](https://github.com/tokio-rs/tokio/blob/master/tokio-util/src/codec/decoder.rs)
- [`Framed`](https://github.com/tokio-rs/tokio/blob/master/tokio-util/src/codec/framed.rs)
- [`FramedImpl`](https://github.com/tokio-rs/tokio/blob/master/tokio-util/src/codec/framed_impl.rs)
### io
Some `no_std` implementations of [`tokio::io`](https://github.com/tokio-rs/tokio/tree/master/tokio/src/io):
- [`AsyncRead`](https://github.com/tokio-rs/tokio/blob/master/tokio/src/io/async_read.rs)
- [`AsyncWrite`](https://github.com/tokio-rs/tokio/blob/master/tokio/src/io/async_write.rs)
- [`ReadBuf`](https://github.com/tokio-rs/tokio/blob/master/tokio/src/io/read_buf.rs)

It also contains a `no_std` implementation of [`std::io::IoSlice`](https://doc.rust-lang.org/std/io/struct.IoSlice.html).
### tcp

## Features
### Default
