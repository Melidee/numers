use thiserror::Error;

use crate::parser::ParseToken;

#[derive(Error, Debug)]
pub enum CompileError {
    #[error("found invalid character: {0}")]
    InvalidCharacter(char),
    #[error("invalid identifier used for assignment")]
    InvalidAssignment,
    #[error("invalid token found in RPN list")]
    InvalidToken(ParseToken),
    #[error("not enough operands in stack for operator")]
    OperandError,
    #[error("name not found: {0}")]
    NameError(String),
}
