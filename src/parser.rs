use crate::expression::Expr;
use crate::statement::{FuncStmt, Stmt};
use crate::token::Token;
use crate::token::TokenType;
use crate::token::TokenType::{
    And, Bang, BangEqual, Comma, Else, Equal, EqualEqual, False, For, Fun, Greater, GreaterEqual,
    Identifier, If, LeftBrace, LeftParen, Less, LessEqual, Minus, Nil, Number, Or, Plus, Print,
    Return, RightBrace, RightParen, Semicolon, Slash, Star, True, Var, While, EOF,
};
use crate::value::LoxError;
use std::mem::discriminant;

pub struct Parser<'a> {
    tokens: &'a [Token],
    current: usize,
}

impl<'a> Parser<'a> {
    pub fn parse(tokens: &[Token]) -> Result<Vec<Stmt>, Vec<LoxError>> {
        let mut parser = Parser::new(tokens);
        parser.parse_statements().map_err(|err| vec![err])
    }

    pub fn parse_statements(&mut self) -> Result<Vec<Stmt>, LoxError> {
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
    fn expression(&mut self) -> Result<Expr, LoxError> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr, LoxError> {
        let expr = self.logic_or()?;
        if self.match_type(&[Equal]) {
            let equals = self.previous().clone();
            let value = self.assignment()?;
            if let Expr::Variable(token) = expr {
                return Ok(Expr::Assignment(token, Box::from(value)));
            }
            return Err(self.format_error(&equals, "Invalid assignment target."));
        }
        Ok(expr)
    }

    fn logic_or(&mut self) -> Result<Expr, LoxError> {
        let mut expr = self.logic_and()?;

        while self.match_type(&[Or]) {
            let operator = self.previous().clone();
            let right = self.logic_and()?;
            expr = Expr::Logical(Box::from(expr), operator, Box::from(right));
        }
        Ok(expr)
    }

    fn logic_and(&mut self) -> Result<Expr, LoxError> {
        let mut expr = self.equality()?;

        while self.match_type(&[And]) {
            let operator = self.previous().clone();
            let right = self.equality()?;
            expr = Expr::Logical(Box::from(expr), operator, Box::from(right));
        }
        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expr, LoxError> {
        self.parse_binary_expression(Parser::comparison, &[EqualEqual, BangEqual])
    }

    fn comparison(&mut self) -> Result<Expr, LoxError> {
        self.parse_binary_expression(Parser::addition, &[Less, LessEqual, Greater, GreaterEqual])
    }

    fn addition(&mut self) -> Result<Expr, LoxError> {
        self.parse_binary_expression(Parser::multiplication, &[Plus, Minus])
    }

    fn multiplication(&mut self) -> Result<Expr, LoxError> {
        self.parse_binary_expression(Parser::unary, &[Slash, Star])
    }

    fn unary(&mut self) -> Result<Expr, LoxError> {
        if self.match_type(&[Bang, Minus]) {
            let operator = self.previous().clone();
            return self
                .unary()
                .map(|right| Expr::Unary(operator, Box::from(right)));
        }
        self.call()
    }

    fn call(&mut self) -> Result<Expr, LoxError> {
        let mut expr = self.primary()?;

        loop {
            if self.match_type(&[LeftParen]) {
                expr = self.finish_call(expr)?;
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn finish_call(&mut self, expr: Expr) -> Result<Expr, LoxError> {
        let mut arguments: Vec<Expr> = vec![];

        if !self.check(&RightParen) {
            loop {
                if arguments.len() >= 255 {
                    return Err(
                        self.format_error(self.peek(), "Cannot have more than 255 arguments")
                    );
                }
                arguments.push(self.expression()?);
                if !self.match_type(&[Comma]) {
                    break;
                }
            }
        }
        let paren = self.consume(&RightParen, "Expected ')' after arguments.")?;
        Ok(Expr::Call(Box::from(expr), paren, arguments))
    }

    fn primary(&mut self) -> Result<Expr, LoxError> {
        if self.match_type(&[False]) {
            return Ok(Expr::False);
        }
        if self.match_type(&[True]) {
            return Ok(Expr::True);
        }
        if self.match_type(&[Nil]) {
            return Ok(Expr::Nil);
        }

        if self.match_type(&[Number(0.0), TokenType::String(String::from(""))]) {
            return match &self.previous().token_type {
                Number(num) => Ok(Expr::Number(*num)),
                TokenType::String(string) => Ok(Expr::String(string.clone())),
                _ => panic!(),
            };
        }

        if self.match_type(&[LeftParen]) {
            let expr = self.expression()?;
            self.consume(&RightParen, "Expected ')' after expression.")?;
            return Ok(Expr::Grouping(Box::from(expr)));
        }

        if self.match_type(&[Identifier("".to_string())]) {
            return Ok(Expr::Variable(self.previous().clone()));
        }

        Err(self.format_error(self.peek(), "Expected expression."))
    }

    fn parse_binary_expression<F>(
        &mut self,
        mut subexpression: F,
        token_types: &[TokenType],
    ) -> Result<Expr, LoxError>
    where
        F: FnMut(&mut Parser<'a>) -> Result<Expr, LoxError>,
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
        discriminant(&self.peek().token_type) == discriminant(&EOF)
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    fn consume(&mut self, token_type: &TokenType, message: &str) -> Result<Token, LoxError> {
        if self.check(token_type) {
            return Ok(self.advance().clone());
        }
        Err(self.format_error(self.peek(), message))
    }

    fn format_error(&self, token: &Token, message: &str) -> LoxError {
        if let EOF = &token.token_type {
            LoxError::Standard(format!(
                "Unexpected EOF at line {}. {} ",
                token.line, message
            ))
        } else {
            LoxError::Standard(format!(
                "{:?} at line {}. {}",
                token.token_type, token.line, message
            ))
        }
    }
}

// statements
impl<'a> Parser<'a> {
    fn declaration(&mut self) -> Result<Stmt, LoxError> {
        if self.match_type(&[Fun]) {
            return self.function_declaration("function");
        }
        if self.match_type(&[Var]) {
            return self.var_declaration();
        }
        self.statement()
    }

    fn function_declaration(&mut self, kind: &str) -> Result<Stmt, LoxError> {
        let name = self.consume(
            &Identifier("".to_string()),
            &format! {"Expected {} name", kind},
        )?;
        self.consume(&LeftParen, &format! {"Expected '(' after {} name", kind})?;
        let mut params: Vec<Token> = vec![];
        if !self.check(&RightParen) {
            loop {
                if params.len() >= 255 {
                    return Err(
                        self.format_error(self.peek(), "Cannot have more than 255 parameters.")
                    );
                }
                params.push(self.consume(&Identifier("".to_string()), "Expected parameter name.")?);
                if !self.match_type(&[Comma]) {
                    break;
                }
            }
        }
        self.consume(&RightParen, "Expected ')' after parameters")?;

        self.consume(&LeftBrace, &format!("Expected '{{' before {} body.", kind))?;
        let body = self.block()?;
        Ok(Stmt::Function(FuncStmt { name, params, body }))
    }

    fn var_declaration(&mut self) -> Result<Stmt, LoxError> {
        let name = self.consume(&Identifier("".to_string()), "Expected variable name")?;
        let initializer = if self.match_type(&[Equal]) {
            Some(self.expression()?)
        } else {
            Option::None
        };
        self.consume(&Semicolon, "Expected ';' after variable declaration")?;
        Ok(Stmt::Var(name, initializer))
    }

    fn statement(&mut self) -> Result<Stmt, LoxError> {
        if self.match_type(&[If]) {
            return self.if_statement();
        }
        if self.match_type(&[Print]) {
            return self.print_statement();
        }
        if self.match_type(&[LeftBrace]) {
            return self.block().map(Stmt::Block);
        }
        if self.match_type(&[While]) {
            return self.while_statement();
        }
        if self.match_type(&[For]) {
            return self.for_statement();
        }
        if self.match_type(&[Return]) {
            return self.return_statement();
        }
        self.expression_statement()
    }

    fn if_statement(&mut self) -> Result<Stmt, LoxError> {
        self.consume(&LeftParen, "Expected '(' after 'if'.")?;
        let condition = self.expression()?;
        self.consume(&RightParen, "Expected ')' after if condition.")?;

        let then_branch = Box::from(self.statement()?);
        let else_branch = if self.match_type(&[Else]) {
            Option::from(Box::from(self.statement()?))
        } else {
            Option::None
        };

        Ok(Stmt::If(condition, then_branch, else_branch))
    }

    fn print_statement(&mut self) -> Result<Stmt, LoxError> {
        let value = self.expression()?;
        self.consume(&Semicolon, "Expected ';' after value.")?;
        Ok(Stmt::Print(value))
    }

    fn block(&mut self) -> Result<Vec<Stmt>, LoxError> {
        let mut statements: Vec<Stmt> = vec![];
        while !self.check(&RightBrace) {
            statements.push(self.declaration()?);
        }
        self.consume(&RightBrace, "Expected '}' afted block.")?;
        Ok(statements)
    }

    fn while_statement(&mut self) -> Result<Stmt, LoxError> {
        self.consume(&LeftParen, "Expected '(' after 'while'.")?;
        let condition = self.expression()?;
        self.consume(&RightParen, "Expected ')' after while condition.")?;

        let body = Box::from(self.statement()?);
        Ok(Stmt::While(condition, body))
    }

    fn for_statement(&mut self) -> Result<Stmt, LoxError> {
        self.consume(&LeftParen, "Expected '(' after 'for'.")?;

        let mut statements: Vec<Stmt> = vec![];
        if !self.match_type(&[Semicolon]) {
            if self.match_type(&[Var]) {
                statements.push(self.var_declaration()?);
            } else {
                statements.push(self.expression_statement()?);
            }
        }

        let condition = if self.check(&Semicolon) {
            Expr::True
        } else {
            self.expression()?
        };
        self.consume(&Semicolon, "Expected ';' after loop condition.")?;

        let increment = if self.check(&RightParen) {
            Option::None
        } else {
            Option::from(Stmt::Expr(self.expression()?))
        };
        self.consume(&RightParen, "Expected ')' after 'for'.")?;

        let mut body = self.statement()?;
        if let Some(increment) = increment {
            body = Stmt::Block(vec![body, increment]);
        }
        statements.push(Stmt::While(condition, Box::from(body)));

        Ok(Stmt::Block(statements))
    }

    fn expression_statement(&mut self) -> Result<Stmt, LoxError> {
        let expression = self.expression()?;
        self.consume(&Semicolon, "Expected ';' after expression.")?;
        Ok(Stmt::Expr(expression))
    }

    fn return_statement(&mut self) -> Result<Stmt, LoxError> {
        let value = if self.check(&Semicolon) {
            None
        } else {
            Some(self.expression()?)
        };
        self.consume(&Semicolon, "Expected ';' after return value.")?;
        Ok(Stmt::Ret(value))
    }
}
