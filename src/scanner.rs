use crate::token::Token;
use crate::token::TokenType;

pub struct Scanner {
    source: Vec<char>,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
    errors: Vec<ScannerError>,
}

impl Scanner {
    pub fn scan(source: &str) -> Result<Vec<Token>, Vec<ScannerError>> {
        let mut scanner = Scanner::new(source);
        scanner.scan_tokens();
        if scanner.errors.is_empty() {
            Ok(scanner.tokens.to_vec())
        } else {
            Err(scanner.errors)
        }
    }

    fn new(source: &str) -> Scanner {
        Scanner {
            source: source.chars().collect(),
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
            errors: Vec::new(),
        }
    }

    fn scan_tokens(&mut self) {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token();
        }
        self.tokens.push(Token {
            token_type: TokenType::EOF,
            line: self.line,
        });
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn scan_token(&mut self) {
        let c = self.advance();

        let mut matches_equal = |equal, not_equal| {
            if self.match_char('=') {
                self.add_token(equal)
            } else {
                self.add_token(not_equal)
            }
        };

        match c {
            '(' => self.add_token(TokenType::LeftParen),
            ')' => self.add_token(TokenType::RightParen),
            '{' => self.add_token(TokenType::LeftBrace),
            '}' => self.add_token(TokenType::RightBrace),
            ',' => self.add_token(TokenType::Comma),
            '.' => self.add_token(TokenType::Dot),
            '-' => self.add_token(TokenType::Minus),
            '+' => self.add_token(TokenType::Plus),
            ';' => self.add_token(TokenType::Semicolon),
            '*' => self.add_token(TokenType::Star),
            '!' => matches_equal(TokenType::BangEqual, TokenType::Bang),
            '=' => matches_equal(TokenType::EqualEqual, TokenType::Equal),
            '<' => matches_equal(TokenType::LessEqual, TokenType::Less),
            '>' => matches_equal(TokenType::GreaterEqual, TokenType::Greater),
            '/' => {
                if self.match_char('/') {
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else {
                    self.add_token(TokenType::Slash)
                }
            }
            ' ' | '\r' | '\t' => (),
            '\n' => self.line += 1,
            '"' => self.string(),
            _ => {
                if c.is_ascii_digit() {
                    self.number()
                } else if c.is_ascii_alphabetic() || c == '_' {
                    self.identifier()
                } else {
                    self.errors.push(ScannerError {
                        line: self.line,
                        error_type: ScannerErrorType::UnexpectedCharacter,
                    });
                }
            }
        }
    }

    fn match_char(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }
        if self.current_char() != expected {
            return false;
        }

        self.current += 1;
        true
    }

    fn advance(&mut self) -> char {
        self.current += 1;
        self.current_char()
    }

    fn add_token(&mut self, token: TokenType) {
        self.tokens.push(Token {
            token_type: token,
            line: self.line,
        });
    }

    fn current_char(&self) -> char {
        self.source[self.current - 1]
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            return '\0';
        }
        self.source[self.current]
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() {
            return '\0';
        }
        self.source[self.current + 1]
    }

    fn string(&mut self) {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            self.errors.push(ScannerError {
                line: self.line,
                error_type: ScannerErrorType::UnterminatedString,
            });
            return;
        }

        self.advance();

        let value = self.source[self.start + 1..self.current - 1]
            .iter()
            .cloned()
            .collect::<String>();
        self.add_token(TokenType::String_(value));
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
        let string_value = &self.source[self.start..self.current]
            .iter()
            .collect::<String>();
        let value: f64 = string_value.parse().unwrap();
        self.add_token(TokenType::Number(value));
    }

    fn identifier(&mut self) {
        while self.peek().is_ascii_alphanumeric() {
            self.advance();
        }
        let value = &self.source[self.start..self.current]
            .iter()
            .collect::<String>();

        if let Some(token_type) = Scanner::default_identifier(value) {
            self.add_token(token_type);
        } else {
            self.add_token(TokenType::Identifier(value.to_string()));
        }
    }

    fn default_identifier(identifier: &str) -> Option<TokenType> {
        match identifier {
            "and" => Some(TokenType::And),
            "class" => Some(TokenType::Class),
            "else" => Some(TokenType::Else),
            "false" => Some(TokenType::False),
            "for" => Some(TokenType::For),
            "fun" => Some(TokenType::Fun),
            "if" => Some(TokenType::If),
            "nil" => Some(TokenType::Nil),
            "or" => Some(TokenType::Or),
            "print" => Some(TokenType::Print),
            "return" => Some(TokenType::Return),
            "super" => Some(TokenType::Super),
            "this" => Some(TokenType::This),
            "true" => Some(TokenType::True),
            "var" => Some(TokenType::Var),
            "while" => Some(TokenType::While),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ScannerError {
    line: usize,
    error_type: ScannerErrorType,
}

#[derive(Debug, Clone)]
enum ScannerErrorType {
    UnexpectedCharacter,
    UnterminatedString,
}

impl std::fmt::Display for ScannerError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match &self.error_type {
            ScannerErrorType::UnterminatedString => {
                write!(f, "unterminated string at line {}", &self.line)
            }
            ScannerErrorType::UnexpectedCharacter => {
                write!(f, "unexpected character at line {}", &self.line)
            }
        }
    }
}

impl std::error::Error for ScannerError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}
