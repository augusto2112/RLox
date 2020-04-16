use crate::expression::Expr;
use crate::statement::Stmt;
use crate::token::Token;
use crate::token::TokenType;
use std::mem::discriminant;

// Grammar:
// expression     → equality ;
// equality       → comparison ( ( "!=" | "==" ) comparison )* ;
// comparison     → addition ( ( ">" | ">=" | "<" | "<=" ) addition )* ;
// addition       → multiplication ( ( "-" | "+" ) multiplication )* ;
// multiplication → unary ( ( "/" | "*" ) unary )* ;
// unary          → ( "!" | "-" ) unary
//                | primary ;
// primary        → NUMBER | STRING | "false" | "true" | "nil"
//                | "(" expression ")" ;

pub struct Parser<'a> {
    tokens: &'a [Token],
    current: usize,
}

impl<'a> Parser<'a> {
    pub fn parse(tokens: &[Token]) -> Result<Vec<Stmt>, Vec<String>> {
        let mut parser = Parser::new(tokens);
        parser.parse_statements().map_err(|err| vec![err])
    }

    pub fn parse_statements(&mut self) -> Result<Vec<Stmt>, String> {
        let mut statements: Vec<Stmt> = vec![];
        while !self.is_at_end() {
            statements.push(self.declaration()?);
        }
        Ok(statements)
    }

    fn new(tokens: &[Token]) -> Parser {
        Parser { tokens, current: 0 }
    }
}

// Expressions
impl<'a> Parser<'a> {
    fn expression(&mut self) -> Result<Expr, String> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr, String> {
        let expr = self.equality()?;
        if self.match_type(&[TokenType::Equal]) {
            let equals = self.previous().clone();
            let value = self.assignment()?;
            if let Expr::Variable(token) = expr {
                return Ok(Expr::Assignment(token, Box::from(value)));
            }
            return Err(self.format_error(&equals, "Invalid assignment target."));
        }
        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expr, String> {
        self.parse_binary_expression(
            Parser::comparison,
            &[TokenType::EqualEqual, TokenType::BangEqual],
        )
    }

    fn comparison(&mut self) -> Result<Expr, String> {
        self.parse_binary_expression(
            Parser::addition,
            &[
                TokenType::Less,
                TokenType::LessEqual,
                TokenType::Greater,
                TokenType::GreaterEqual,
            ],
        )
    }

    fn addition(&mut self) -> Result<Expr, String> {
        self.parse_binary_expression(Parser::multiplication, &[TokenType::Plus, TokenType::Minus])
    }

    fn multiplication(&mut self) -> Result<Expr, String> {
        self.parse_binary_expression(Parser::unary, &[TokenType::Slash, TokenType::Star])
    }

    fn unary(&mut self) -> Result<Expr, String> {
        if self.match_type(&[TokenType::Bang, TokenType::Minus]) {
            let operator = self.previous().clone();
            return self
                .unary()
                .map(|right| Expr::Unary(operator, Box::from(right)));
        }
        self.primary()
    }

    fn primary(&mut self) -> Result<Expr, String> {
        if self.match_type(&[TokenType::False]) {
            return Ok(Expr::False);
        }
        if self.match_type(&[TokenType::True]) {
            return Ok(Expr::True);
        }
        if self.match_type(&[TokenType::Nil]) {
            return Ok(Expr::Nil);
        }

        if self.match_type(&[TokenType::Number(0.0), TokenType::String(String::from(""))]) {
            return match &self.previous().token_type {
                TokenType::Number(num) => Ok(Expr::Number(*num)),
                TokenType::String(string) => Ok(Expr::String(string.clone())),
                _ => panic!(),
            };
        }

        if self.match_type(&[TokenType::LeftParen]) {
            let expr = self.expression()?;
            self.consume(&TokenType::RightParen, "Expected ')' after expression.")?;
            return Ok(Expr::Grouping(Box::from(expr)));
        }

        if self.match_type(&[TokenType::Identifier("".to_string())]) {
            return Ok(Expr::Variable(self.previous().clone()));
        }

        Err(self.format_error(self.peek(), "Expected expression."))
    }

    fn parse_binary_expression<F>(
        &mut self,
        mut subexpression: F,
        token_types: &[TokenType],
    ) -> Result<Expr, String>
    where
        F: FnMut(&mut Parser<'a>) -> Result<Expr, String>,
    {
        let mut expr = subexpression(self)?;

        while self.match_type(token_types) {
            let operator = self.previous().clone();
            let right = subexpression(self)?;
            expr = Expr::Binary(Box::from(expr), operator, Box::from(right));
        }

        Ok(expr)
    }

    fn match_type(&mut self, types: &[TokenType]) -> bool {
        for token_type in types {
            if self.check(token_type) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn check(&self, token_type: &TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }
        discriminant(&self.peek().token_type) == discriminant(token_type)
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        &self.previous()
    }

    fn is_at_end(&self) -> bool {
        discriminant(&self.peek().token_type) == discriminant(&TokenType::EOF)
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    fn consume(&mut self, token_type: &TokenType, message: &str) -> Result<Token, String> {
        if self.check(token_type) {
            return Ok(self.advance().clone());
        }
        Err(self.format_error(self.peek(), message))
    }

    fn format_error(&self, token: &Token, message: &str) -> String {
        match &token.token_type {
            TokenType::EOF => format!("Unexpected EOF at line {}. {} ", token.line, message),
            _ => format!("{:?} at line {}. {}", token.token_type, token.line, message),
        }
    }
}

// statements
impl<'a> Parser<'a> {
    fn declaration(&mut self) -> Result<Stmt, String> {
        if self.match_type(&[TokenType::Var]) {
            return self.var_declaration();
        }
        self.statement()
    }

    fn var_declaration(&mut self) -> Result<Stmt, String> {
        let name = self.consume(
            &TokenType::Identifier("".to_string()),
            "Expected variable name",
        )?;
        let initializer = if self.match_type(&[TokenType::Equal]) {
            Some(self.expression()?)
        } else {
            Option::None
        };
        self.consume(
            &TokenType::Semicolon,
            "Expected ';' after variable declaration",
        )?;
        Ok(Stmt::Var(name, initializer))
    }

    fn statement(&mut self) -> Result<Stmt, String> {
        if self.match_type(&[TokenType::Print]) {
            return self.print_statement()
        }
        if self.match_type(&[TokenType::LeftBrace]) {
            return self.block().map(Stmt::Block)
        }
        self.expression_statement()
    }

    fn print_statement(&mut self) -> Result<Stmt, String> {
        let value = self.expression()?;
        self.consume(&TokenType::Semicolon, "Expected ';' after value.")?;
        Ok(Stmt::Print(value))
    }

    fn block(&mut self) -> Result<Vec<Stmt>, String> {
        let mut statements: Vec<Stmt> = vec![];
        while !self.check(&TokenType::RightBrace) {
            statements.push(self.declaration()?);
        }
        self.consume(&TokenType::RightBrace, "Expected '}' afted block.")?;
        Ok(statements)
    }

    fn expression_statement(&mut self) -> Result<Stmt, String> {
        let expression = self.expression()?;
        self.consume(&TokenType::Semicolon, "Expected ';' after expression.")?;
        Ok(Stmt::Expr(expression))
    }
}
