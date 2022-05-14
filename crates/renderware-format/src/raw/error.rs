use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("parsing error")]
    NomError(#[from] nom::error::Error<super::UnparsedData>),
    #[error("IO error")]
    IoError(#[from] std::io::Error),
}
