#[derive(Clone, Debug, PartialEq)]
pub enum TokenType {
    // Single-character tokens.
    LeftParen,
    RightParen,
    // Math operators.
    Plus,
    Minus,
    Star,
    Slash,
    Modulo,
    Caret,
    Comma,
    Dot,
    Identifier,
    Number,
    Eof,
}

#[derive(Debug)]
pub struct Token {
    pub kind: TokenType,
    pub lexeme: String,
    pub literal: Option<f64>,
    pub line: usize,
}

impl Clone for Token {
    fn clone(&self) -> Self {
        Token {
            kind: self.kind.clone(),
            lexeme: self.lexeme.clone(),
            literal: self.literal,
            line: self.line,
        }
    }
}

pub struct Scanner {
    source: String,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
}

impl Scanner {
    pub fn new(source: String) -> Scanner {
        Scanner {
            source,
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub fn scan_tokens(&mut self) -> Vec<Token> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token();
        }

        self.tokens.push(Token {
            kind: TokenType::Eof,
            lexeme: "".to_string(),
            literal: None,
            line: self.line,
        });

        (*self.tokens).to_vec()
    }

    fn scan_token(&mut self) {
        let c = self.advance();
        match c {
            '(' => self.add_token(TokenType::LeftParen),
            ')' => self.add_token(TokenType::RightParen),
            '+' => self.add_token(TokenType::Plus),
            '-' => self.add_token(TokenType::Minus),
            '*' => self.add_token(TokenType::Star),
            '/' => self.add_token(TokenType::Slash),
            '%' => self.add_token(TokenType::Modulo),
            '^' => self.add_token(TokenType::Caret),
            ',' => self.add_token(TokenType::Comma),
            '.' => self.add_token(TokenType::Dot),
            ' ' | '\r' | '\t' => (),
            char => {
                if char.is_ascii_digit() {
                    self.number();
                } else if char.is_alphabetic() {
                    self.identifier();
                }
            }
        }
    }

    fn identifier(&mut self) {
        while self.peek().is_alphanumeric() {
            self.advance();
        }

        self.add_token(TokenType::Identifier);
    }

    fn number(&mut self) {
        while self.peek().is_ascii_digit() {
            self.advance();
        }

        if self.peek() == '.' && self.peek_next().is_ascii_digit() {
            self.advance();

            while self.peek().is_ascii_digit() {
                self.advance();
            }
        }

        let number = self.source[self.start..self.current].parse::<f64>().unwrap();
        self.add_token_with_literal(TokenType::Number, number);
    }

    fn advance(&mut self) -> char {
        self.current += 1;
        self.source.chars().nth(self.current - 1).unwrap()
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.source.chars().nth(self.current).unwrap()
        }
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() {
            '\0'
        } else {
            self.source.chars().nth(self.current + 1).unwrap()
        }
    }

    fn add_token(&mut self, kind: TokenType) {
        self.tokens.push(Token {
            kind,
            lexeme: self.source[self.start..self.current].to_string(),
            literal: None,
            line: self.line,
        });
    }

    fn add_token_with_literal(&mut self, kind: TokenType, literal: f64) {
        self.tokens.push(Token {
            kind,
            lexeme: self.source[self.start..self.current].to_string(),
            literal: Some(literal),
            line: self.line,
        });
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }
}
