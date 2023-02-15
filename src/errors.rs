use std::fmt::Display;

use crate::scanner::Token;

#[derive(Debug)]
pub struct CalculatorError {
    pub error: CalculatorErrorType,
    pub token: Option<Token>,
}

impl Display for CalculatorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.token {
            Some(token) => write!(f, "(At '{}' in line {}) {:?}", token.lexeme, token.line, self.error),
            None => write!(f, "{:?}", self.error),
        }
    }
}

#[derive(Debug)]
pub enum CalculatorErrorType {
    SyntaxError(String),
    AdditionalCodeAfterEnd,
    TooManyArguments,
    ExpectedExpression,
    FunctionArityMismatch(String, usize, usize),
    UndefinedVariableOrFunction(String),
}
