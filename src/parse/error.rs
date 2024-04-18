use std::fmt::Display;

use rusty_money::MoneyError;

#[derive(Debug)]
pub enum Error {
    HtmlParse(String),
    TextNodeParse(String),
    PriceParse(String),
    Http(String),
    Internal(String),
}

impl From<MoneyError> for Error {
    fn from(e: MoneyError) -> Self {
        Self::PriceParse(e.to_string())
    }
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Self::Http(e.to_string())
    }
}

impl Error {
    pub fn html_parse_error(msg: &str) -> Self {
        Self::HtmlParse(msg.to_string())
    }
    pub fn text_node_parse_error(msg: &str) -> Self {
        Self::TextNodeParse(msg.to_string())
    }
    pub fn price_parse_error(msg: &str) -> Self {
        Self::PriceParse(msg.to_string())
    }

    pub fn http_error(msg: &str) -> Self {
        Self::Http(msg.to_string())
    }
}

impl Error {
    pub fn internal_error(msg: &str) -> Self {
        Self::Internal(msg.to_string())
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::HtmlParse(msg) => write!(f, "HTML Parse Error: {}", msg),
            Self::TextNodeParse(msg) => write!(f, "Text Node Parse Error: {}", msg),
            Self::PriceParse(msg) => write!(f, "Price Parse Error: {}", msg),
            Self::Http(msg) => write!(f, "HTTP Request Error: {msg}"),
            Self::Internal(msg) => write!(f, "Internal Error: {msg}"),
        }
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;
