use std::{error::Error, fmt::Display};

#[derive(Debug, PartialEq, Eq)]
pub enum ClientError {
    FailedToConnect,
    FailedToCreateServer,
}

impl Display for ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for ClientError {}
