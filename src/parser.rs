use crate::{
    errors::{CalculatorError, CalculatorErrorType},
    scanner::{Token, TokenType},
};
use core::fmt::Debug;

#[derive(Clone)]
pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

pub mod expressions {
    use std::fmt::Formatter;

    use super::*;
    pub trait Expression: Debug {
        fn accept(&self, visitor: &mut dyn Visitor) -> Result<f64, CalculatorError>;
    }

    pub trait Visitor {
        fn visit_binary_expr(&mut self, expr: &Binary) -> Result<f64, CalculatorError>;
        fn visit_grouping_expr(&mut self, expr: &Grouping) -> Result<f64, CalculatorError>;
        fn visit_literal_expr(&mut self, expr: &Literal) -> Result<f64, CalculatorError>;
        fn visit_unary_expr(&mut self, expr: &Unary) -> Result<f64, CalculatorError>;
        fn visit_call_expr(&mut self, expr: &Call) -> Result<f64, CalculatorError>;
        fn visit_variable_expr(&mut self, expr: &Variable) -> Result<f64, CalculatorError>;
    }

    pub struct Binary {
        pub left: Box<dyn Expression>,
        pub operator: Token,
        pub right: Box<dyn Expression>,
    }
    impl Expression for Binary {
        fn accept(&self, visitor: &mut dyn Visitor) -> Result<f64, CalculatorError> {
            visitor.visit_binary_expr(self)
        }
    }
    impl Debug for Binary {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(
                f,
                "({} {:?} {:?})",
                self.operator.lexeme, self.left, self.right
            )
        }
    }

    pub struct Grouping {
        pub expression: Box<dyn Expression>,
    }
    impl Expression for Grouping {
        fn accept(&self, visitor: &mut dyn Visitor) -> Result<f64, CalculatorError> {
            visitor.visit_grouping_expr(self)
        }
    }
    impl Debug for Grouping {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "({:?})", self.expression)
        }
    }

    pub struct Literal {
        pub value: Token,
    }
    impl Expression for Literal {
        fn accept(&self, visitor: &mut dyn Visitor) -> Result<f64, CalculatorError> {
            visitor.visit_literal_expr(self)
        }
    }
    impl Debug for Literal {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.value.lexeme)
        }
    }

    pub struct Unary {
        pub operator: Token,
        pub right: Box<dyn Expression>,
    }
    impl Expression for Unary {
        fn accept(&self, visitor: &mut dyn Visitor) -> Result<f64, CalculatorError> {
            visitor.visit_unary_expr(self)
        }
    }
    impl Debug for Unary {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "({} {:?})", self.operator.lexeme, self.right)
        }
    }

    pub struct Call {
        pub callee: Token,
        pub paren: Token,
        pub arguments: Vec<Box<dyn Expression>>,
    }
    impl Expression for Call {
        fn accept(&self, visitor: &mut dyn Visitor) -> Result<f64, CalculatorError> {
            visitor.visit_call_expr(self)
        }
    }
    impl Debug for Call {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "({} {:?})", self.callee.lexeme, self.arguments)
        }
    }

    pub struct Variable {
        pub name: Token,
    }
    impl Expression for Variable {
        fn accept(&self, visitor: &mut dyn Visitor) -> Result<f64, CalculatorError> {
            visitor.visit_variable_expr(self)
        }
    }
    impl Debug for Variable {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.name.lexeme)
        }
    }
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        Parser { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Result<Box<dyn expressions::Expression>, CalculatorError> {
        let expr = self.expression()?;
        if !self.is_at_end() {
            return Err(CalculatorError {
                error: CalculatorErrorType::AdditionalCodeAfterEnd,
                token: Some(self.peek()),
            });
        }
        Ok(expr)
    }

    fn expression(&mut self) -> Result<Box<dyn expressions::Expression>, CalculatorError> {
        self.addition()
    }

    fn addition(&mut self) -> Result<Box<dyn expressions::Expression>, CalculatorError> {
        let mut expr = self.multiplication()?;

        while self.match_token(&[TokenType::Plus, TokenType::Minus]) {
            let operator = self.previous();
            let right = self.multiplication()?;
            expr = Box::new(expressions::Binary {
                left: expr,
                operator,
                right,
            });
        }

        Ok(expr)
    }

    fn multiplication(&mut self) -> Result<Box<dyn expressions::Expression>, CalculatorError> {
        let mut expr = self.power()?;

        while self.match_token(&[TokenType::Star, TokenType::Slash]) {
            let operator = self.previous();
            let right = self.power()?;
            expr = Box::new(expressions::Binary {
                left: expr,
                operator,
                right,
            });
        }

        Ok(expr)
    }

    fn power(&mut self) -> Result<Box<dyn expressions::Expression>, CalculatorError> {
        let mut expr = self.unary()?;

        while self.match_token(&[TokenType::Caret]) {
            let operator = self.previous();
            let right = self.unary()?;
            expr = Box::new(expressions::Binary {
                left: expr,
                operator,
                right,
            });
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Box<dyn expressions::Expression>, CalculatorError> {
        if self.match_token(&[TokenType::Minus, TokenType::Plus]) {
            let operator = self.previous();
            let right = self.expression()?;
            return Ok(Box::new(expressions::Unary { operator, right }));
        }

        self.primary()
    }

    fn primary(&mut self) -> Result<Box<dyn expressions::Expression>, CalculatorError> {
        let mut expr: Box<dyn expressions::Expression>;
        if self.match_token(&[TokenType::Number]) {
            expr = Box::new(expressions::Literal {
                value: self.previous(),
            });
        } else if self.match_token(&[TokenType::LeftParen]) {
            expr = self.expression()?;
            self.consume(TokenType::RightParen, "Expected ')' after expression.")?;
            expr = Box::new(expressions::Grouping { expression: expr });
        } else if self.match_token(&[TokenType::Identifier]) {
            let name = self.previous();
            if !self.match_token(&[TokenType::LeftParen]) {
                return Ok(Box::new(expressions::Variable { name }));
            }
            let mut arguments = Vec::new();
            if !self.match_token(&[TokenType::RightParen]) {
                loop {
                    if arguments.len() >= 255 {
                        return Err(self
                            .clone()
                            .create_error(CalculatorErrorType::TooManyArguments));
                    }
                    arguments.push(self.expression()?);
                    if !self.match_token(&[TokenType::Comma]) {
                        break;
                    }
                }
                self.consume(TokenType::RightParen, "Expected ')' after arguments.")?;
            }
            expr = Box::new(expressions::Call {
                callee: name,
                paren: self.previous(),
                arguments,
            });
        } else {
            return Err(self
                .clone()
                .create_error(CalculatorErrorType::ExpectedExpression));
        }

        // if self.match_token(&[TokenType::LeftParen]) {
        //     let to_multiply = self.expression()?;
        //     self.consume(TokenType::RightParen, "Expected ')' after expression.")?;
        //     expr = Box::new(expressions::ParanthesisMultiplication {
        //         left: expr,
        //         right: to_multiply,
        //     });
        // }

        Ok(expr)
    }

    fn create_error(self, error: CalculatorErrorType) -> CalculatorError {
        CalculatorError {
            error,
            token: Some(self.peek()),
        }
    }

    fn consume(&mut self, kind: TokenType, message: &str) -> Result<Token, CalculatorError> {
        if self.check(&kind) {
            return Ok(self.advance());
        }
        Err(self
            .clone()
            .create_error(CalculatorErrorType::SyntaxError(message.to_string())))
    }

    fn match_token(&mut self, kinds: &[TokenType]) -> bool {
        for kind in kinds {
            if self.check(kind) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn check(&self, kind: &TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }
        self.peek().kind == *kind
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn peek(&self) -> Token {
        self.tokens[self.current].clone()
    }

    fn previous(&self) -> Token {
        self.tokens[self.current - 1].clone()
    }

    fn is_at_end(&self) -> bool {
        self.peek().kind == TokenType::Eof
    }
}
