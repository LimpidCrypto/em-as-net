use thiserror_no_std::Error;

#[derive(Error, Debug)]
pub enum TcpStackError {
    #[error("TCP stack stopped.")]
    StackStoppedError,
}
