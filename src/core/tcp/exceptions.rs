use thiserror_no_std::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum TcpException {
    #[error("Unable to connect to host")]
    UnableToConnect,
}
