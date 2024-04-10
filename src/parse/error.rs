use std::fmt::Display;

use rusty_money::MoneyError;

#[derive(Debug)]
pub enum Error {
    HTMLParseError(String),
    TextNodeParseError(String),
    PriceParseError(String),
}

impl From<MoneyError> for Error {
    fn from(e: MoneyError) -> Self {
        Self::PriceParseError(e.to_string())
    }
}

impl Error {
    pub fn html_parse_error(msg: &str) -> Self {
        Self::HTMLParseError(msg.to_string())
    }
    pub fn text_node_parse_error(msg: &str) -> Self {
        Self::TextNodeParseError(msg.to_string())
    }
    pub fn price_parse_error(msg: &str) -> Self {
        Self::PriceParseError(msg.to_string())
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::HTMLParseError(msg) => write!(f, "HTML Parse Error: {}", msg),
            Self::TextNodeParseError(msg) => write!(f, "Text Node Parse Error: {}", msg),
            Self::PriceParseError(msg) => write!(f, "Price Parse Error: {}", msg),
        }
    }
}

impl std::error::Error for Error {}
