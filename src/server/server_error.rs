use std::{error::Error, fmt::Display};

#[derive(Debug, PartialEq, Eq)]
pub enum ServerError {
    FailedToBind,
    FailedToAccept,
    FailedToSerialize,
}

impl Display for ServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for ServerError {}
