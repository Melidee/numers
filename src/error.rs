use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("found invalid character: {0}")]
    InvalidCharacter(char)
}