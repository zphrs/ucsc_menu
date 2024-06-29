use firestore::errors::FirestoreError;

use crate::parse;
use std::fmt::{self, Display, Formatter};

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Parse(parse::Error),
    Request(reqwest::Error),
    Database(FirestoreError),
    Json(serde_json::Error),
    Io(std::io::Error),
}

impl From<parse::Error> for Error {
    fn from(e: parse::Error) -> Self {
        Self::Parse(e)
    }
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Self::Request(e)
    }
}

impl From<FirestoreError> for Error {
    fn from(e: FirestoreError) -> Self {
        Self::Database(e)
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e)
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parse(e) => write!(f, "Parse error: {e}"),
            Self::Request(e) => write!(f, "Request error: {e}"),
            Self::Database(e) => write!(f, "Database error: {e}"),
            Self::Json(e) => write!(f, "Json error: {e}"),
            Self::Io(e) => write!(f, "IO error: {e}"),
        }
    }
}

impl std::error::Error for Error {}
