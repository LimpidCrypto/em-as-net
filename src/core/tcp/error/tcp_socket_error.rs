use thiserror_no_std::Error;

#[derive(Error, Debug)]
pub enum TcpSocketError {
    #[error("Failed to write `buf`")]
    WriteError,
}
