use thiserror_no_std::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ReadBufException {}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum AsyncReadException {
    UnableToRead
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum AsyncWriteException {
    UnableToWrite,
    UnableToFlush,
    UnableToClose,
}
