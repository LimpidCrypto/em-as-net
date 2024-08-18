#[cfg(not(feature = "std"))]
pub use _embassy::*;
#[cfg(feature = "std")]
pub use _tokio::*;

#[cfg(not(feature = "std"))]
mod _embassy {
    use embassy_net::tcp::TcpSocket as EmbassyTcpSocket;

    pub type TcpSocket<'a> = EmbassyTcpSocket<'a>;
}

#[cfg(feature = "std")]
mod _tokio {
    use embedded_io_adapters::tokio_1::FromTokio;
    use tokio::net::{TcpListener as TokioTcpListener, TcpStream as TokioTcpStream};
    pub type TcpStream = FromTokio<TokioTcpStream>;
    pub type TcpListener = FromTokio<TokioTcpListener>;
}
