use firestore::errors::FirestoreError;

use crate::parse;
use std::fmt::{self, Display, Formatter};

#[derive(Debug)]
pub enum Error {
    Parse(parse::Error),
    Request(reqwest::Error),
    Database(FirestoreError),
    Json(serde_json::Error),
}

impl From<parse::Error> for Error {
    fn from(e: parse::Error) -> Self {
        Error::Parse(e)
    }
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Error::Request(e)
    }
}

impl From<FirestoreError> for Error {
    fn from(e: FirestoreError) -> Self {
        Error::Database(e)
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::Json(e)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Error::Parse(e) => write!(f, "Parse error: {}", e),
            Error::Request(e) => write!(f, "Request error: {}", e),
            Error::Database(e) => write!(f, "Database error: {}", e),
            Error::Json(e) => write!(f, "Json error: {}", e),
        }
    }
}


pub type Result<T> = std::result::Result<T, Error>;