use {
    crate::{
        config::parser::{DataType, ParseResult, Parser, UnexpectedDataType},
        toml::toml_span::{Span, SpannedExt},
    },
    jay_config::bar::BarLocation,
    thiserror::Error,
};

#[derive(Debug, Error)]
pub enum BarLocationParserError {
    #[error(transparent)]
    Expected(#[from] UnexpectedDataType),
    #[error("Unknown bar location {0}")]
    Unknown(String),
}

pub struct BarLocationParser;

impl Parser for BarLocationParser {
    type Value = BarLocation;
    type Error = BarLocationParserError;
    const EXPECTED: &'static [DataType] = &[DataType::String];

    fn parse_string(&mut self, span: Span, string: &str) -> ParseResult<Self> {
        match string {
            "top" => Ok(BarLocation::Top),
            "bottom" => Ok(BarLocation::Bottom),
            _ => Err(BarLocationParserError::Unknown(string.to_string()).spanned(span)),
        }
    }
}
