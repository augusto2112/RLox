use crate::enviroment::Environment;
use crate::expression::Expr;
use crate::statement::Stmt;
use crate::token::Token;
use crate::token::TokenType;
use crate::value::LoxValue;

pub struct Interpreter {
    enviroment: Environment,
}

impl Interpreter {
    pub fn new() -> Interpreter {
        Interpreter {
            enviroment: Environment::new(),
        }
    }
    pub fn interpret(&mut self, statements: &[Stmt]) -> Result<(), String> {
        for statement in statements {
            self.interpret_statement(statement)?;
        }
        Ok(())
    }

    pub fn interpret_statement(&mut self, statement: &Stmt) -> Result<(), String> {
        match statement {
            Stmt::Print(expression) => {
                let value = self.interpret_expression(expression)?;
                println!("{}", value);
                Ok(())
            }
            Stmt::Expr(expression) => self.interpret_expression(expression).map(|_| {}),
            Stmt::Var(
                Token {
                    token_type: TokenType::Identifier(name),
                    ..
                },
                expression,
            ) => {
                if let Some(expression) = expression {
                    let value = self.interpret_expression(expression)?;
                    self.enviroment.define(name, &value);
                } else {
                    self.enviroment.define(name, &LoxValue::Nil);
                }
                Ok(())
            },
            Stmt::Block(statements) => self.execute_block(&statements),
            statement => panic!("Interpreter bug. Unexpected statement: {:?}", statement)
        }
    }

    pub fn interpret_expression(&mut self, expression: &Expr) -> Result<LoxValue, String> {
        match expression {
            Expr::Number(number) => Ok(LoxValue::Number(*number)),
            Expr::String(string) => Ok(LoxValue::String(string.to_string())),
            Expr::True => Ok(LoxValue::Bool(true)),
            Expr::False => Ok(LoxValue::Bool(false)),
            Expr::Nil => Ok(LoxValue::Nil),
            Expr::Grouping(expression) => self.interpret_expression(expression),
            Expr::Unary(
                Token {
                    token_type: TokenType::Bang,
                    ..
                },
                expression,
            ) => {
                let expression = self.interpret_expression(expression)?;
                Ok(LoxValue::Bool(!expression.is_truthy()))
            }
            Expr::Unary(
                Token {
                    token_type: TokenType::Minus,
                    ..
                },
                expression,
            ) => match self.interpret_expression(expression)? {
                LoxValue::Number(number) => Ok(LoxValue::Number(-number)),
                _ => Err("Error: operand must be a number.".to_string()),
            },
            Expr::Binary(left, Token { token_type, line }, right) => {
                match (self.interpret_expression(left)?, token_type, self.interpret_expression(right)?) {
                    (LoxValue::Number(left), TokenType::Plus, LoxValue::Number(right)) => {
                        Ok(LoxValue::Number(left + right))
                    },
                    (LoxValue::String(left), TokenType::Plus, LoxValue::String(right)) => {
                        Ok(LoxValue::String(left + &right))
                    }
                    (LoxValue::Number(left), TokenType::Minus, LoxValue::Number(right)) => {
                        Ok(LoxValue::Number(left - right))
                    }
                    (LoxValue::Number(left), TokenType::Slash, LoxValue::Number(right)) => {
                        Ok(LoxValue::Number(left / right))
                    }
                    (LoxValue::Number(left), TokenType::Star, LoxValue::Number(right)) => {
                        Ok(LoxValue::Number(left * right))
                    }
                    (LoxValue::Number(left), TokenType::Less, LoxValue::Number(right)) => {
                        Ok(LoxValue::Bool(left < right))
                    }
                    (LoxValue::Number(left), TokenType::LessEqual, LoxValue::Number(right)) => {
                        Ok(LoxValue::Bool(left <= right))
                    }
                    (LoxValue::Number(left), TokenType::Greater, LoxValue::Number(right)) => {
                        Ok(LoxValue::Bool(left > right))
                    }
                    (LoxValue::Number(left), TokenType::GreaterEqual, LoxValue::Number(right)) => {
                        Ok(LoxValue::Bool(left >= right))
                    }
                    (left, TokenType::EqualEqual, right) => Ok(LoxValue::Bool(left == right)),
                    (left, TokenType::BangEqual, right) => Ok(LoxValue::Bool(left != right)),
                    (_, TokenType::Minus, _)
                    | (_, TokenType::Star, _)
                    | (_, TokenType::Slash, _)
                    | (_, TokenType::Less, _)
                    | (_, TokenType::LessEqual, _)
                    | (_, TokenType::Greater, _)
                    | (_, TokenType::GreaterEqual, _) => Err(format!(
                        "Error in line: {}, operands must both be numbers.",
                        line
                    )),
                    (_, TokenType::Plus, _) => Err(format!(
                        "Error in line: {}, operands must both be numbers or strings.",
                        line
                    )),
                    (left, op, right) => panic!("Interpreter bug: Unexpected match of left expression: {:?}, operation: {:?}, right expression: {:?}", left, op, right),
                }
            },
            Expr::Variable(token) => self.enviroment.get(&token),
            Expr::Assignment(Token {
                token_type: TokenType::Identifier(name),
                ..
            }, expression) => {
                let value = self.interpret_expression(expression)?;
                self.enviroment.assign(name, &value)?;
                Ok(value)
            }
            expression => panic!("Interpreter bug: unexpected expression: {:?}", expression),
        }
    }

    fn execute_block(&mut self, statements: &[Stmt]) -> Result<(), String> {
        self.enviroment.add_subenvironment();
        for statement in statements {
            let result = self.interpret_statement(statement);
            if result.is_err() {
                self.enviroment.remove_subenvironment();
                return result
            }
        }   
        self.enviroment.remove_subenvironment(); 
        Ok(())
    }
}
