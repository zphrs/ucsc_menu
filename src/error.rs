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

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parse(e) => write!(f, "Parse error: {e}"),
            Self::Request(e) => write!(f, "Request error: {e}"),
            Self::Database(e) => write!(f, "Database error: {e}"),
            Self::Json(e) => write!(f, "Json error: {e}"),
        }
    }
}
