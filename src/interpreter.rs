use crate::environment::Environment;
use crate::expression::Expr;
use crate::statement::Stmt;
use crate::token::Token;
use crate::token::TokenType;
use crate::value::{Callable, Return};
use crate::value::{LoxError, LoxValue};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::UNIX_EPOCH;

pub struct Interpreter {
    pub global: Rc<RefCell<Environment>>,
    environment: Rc<RefCell<Environment>>,
}

impl Interpreter {
    pub fn new() -> Interpreter {
        let global = Environment::new();
        let func = |_: &mut Interpreter, _: &[LoxValue]| -> Result<LoxValue, LoxError> {
            println!(
                "{:#?}",
                std::time::SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("error")
            );
            Ok(LoxValue::Nil)
        };
        let callable = &LoxValue::Callable(Callable::Native { arity: 0, func });
        global.borrow_mut().define("clock", callable);

        let environment = Rc::clone(&global);
        Interpreter {
            global,
            environment,
        }
    }

    pub fn interpret(&mut self, statements: &[Stmt]) -> Result<(), LoxError> {
        for statement in statements {
            self.interpret_statement(statement)?;
        }
        Ok(())
    }

    pub fn interpret_statement(&mut self, statement: &Stmt) -> Result<(), LoxError> {
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
                    self.environment.borrow_mut().define(name, &value);
                } else {
                    self.environment.borrow_mut().define(name, &LoxValue::Nil);
                }
                Ok(())
            }
            Stmt::Block(statements) => {
                let new = Environment::new_enclosed(Rc::clone(&self.environment));
                self.execute_block(&statements, new)
            }
            Stmt::If(condition, then_block, else_block) => {
                if self.interpret_expression(condition)?.is_truthy() {
                    self.interpret_statement(then_block)?;
                } else if let Some(else_block) = else_block {
                    self.interpret_statement(else_block)?;
                }
                Ok(())
            }
            Stmt::While(condition, body) => {
                while self.interpret_expression(condition)?.is_truthy() {
                    self.interpret_statement(body)?;
                }
                Ok(())
            }
            Stmt::Function(func_stmt) => {
                if let TokenType::Identifier(name) = &func_stmt.name.token_type {
                    let func = LoxValue::Callable(Callable::Function {
                        arity: func_stmt.params.len(),
                        func_stmt: func_stmt.clone(),
                        environment: Rc::clone(&self.environment),
                    });
                    self.environment.borrow_mut().define(name, &func);
                    Ok(())
                } else {
                    panic!("Compiler bug. Unexpected token type: {:?}", &func_stmt.name);
                }
            }
            Stmt::Ret(expr) => {
                let mut value: Option<LoxValue> = None;
                if let Some(expr) = expr {
                    value = Some(self.interpret_expression(expr)?);
                }
                // oh no so ugly!!
                Err(LoxError::Return(Return { value }))
            }
            statement => panic!("Interpreter bug. Unexpected statement: {:?}", statement),
        }
    }

    pub fn interpret_expression(&mut self, expression: &Expr) -> Result<LoxValue, LoxError> {
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
                _ => Err(LoxError::Standard(
                    "Error: operand must be a number.".to_string(),
                )),
            },
            Expr::Binary(left, token, right) => {
                self.interpret_binary_expression(left, token, right)
            }
            Expr::Variable(token) => self.environment.borrow().get(&token),
            Expr::Assignment(
                Token {
                    token_type: TokenType::Identifier(name),
                    ..
                },
                expression,
            ) => {
                let value = self.interpret_expression(expression)?;
                self.environment.borrow_mut().assign(name, &value)?;
                Ok(value)
            }
            Expr::Logical(
                left,
                Token {
                    token_type: TokenType::Or,
                    ..
                },
                right,
            ) => {
                let left = self.interpret_expression(left)?;
                if left.is_truthy() {
                    Ok(left)
                } else {
                    self.interpret_expression(right)
                }
            }
            Expr::Logical(
                left,
                Token {
                    token_type: TokenType::And,
                    ..
                },
                right,
            ) => {
                let left = self.interpret_expression(left)?;
                if left.is_truthy() {
                    self.interpret_expression(right)
                } else {
                    Ok(left)
                }
            }
            Expr::Call(callee, _, arguments) => {
                let callee = self.interpret_expression(callee)?;
                let arguments: Result<Vec<LoxValue>, LoxError> = arguments
                    .iter()
                    .map(|argument| self.interpret_expression(argument))
                    .collect();
                let arguments = arguments?;
                if let LoxValue::Callable(function) = callee {
                    if function.arity() == arguments.len() {
                        function.call(self, &arguments)
                    } else {
                        Err(LoxError::Standard(format!(
                            "Expected {} arguments but got {} .",
                            function.arity(),
                            arguments.len()
                        )))
                    }
                } else {
                    Err(LoxError::Standard(
                        "Can only call functions and classes".to_string(),
                    ))
                }
            }

            expression => panic!("Interpreter bug: unexpected expression: {:?}", expression),
        }
    }

    fn interpret_binary_expression(
        &mut self,
        left: &Expr,
        token: &Token,
        right: &Expr,
    ) -> Result<LoxValue, LoxError> {
        let Token { token_type, line } = token;
        match (self.interpret_expression(left)?, token_type, self.interpret_expression(right)?) {
            (LoxValue::Number(left), TokenType::Plus, LoxValue::Number(right)) => {
                Ok(LoxValue::Number(left + right))
            }
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
            | (_, TokenType::GreaterEqual, _) => Err(LoxError::Standard(format!(
                "Error in line: {}, operands must both be numbers.",
                line
            ))),
            (_, TokenType::Plus, _) => Err(LoxError::Standard(format!(
                "Error in line: {}, operands must both be numbers or strings.",
                line
            ))),
            (left, op, right) => panic!("Interpreter bug: Unexpected match of left expression: {:?}, operation: {:?}, right expression: {:?}", left, op, right),
        }
    }

    pub fn execute_block(
        &mut self,
        statements: &[Stmt],
        new_environment: Rc<RefCell<Environment>>,
    ) -> Result<(), LoxError> {
        let mut old = std::mem::replace(&mut self.environment, new_environment);
        for statement in statements {
            let result = self.interpret_statement(statement);
            if result.is_err() {
                std::mem::swap(&mut self.environment, &mut old);
                return result;
            }
        }
        std::mem::swap(&mut self.environment, &mut old);
        Ok(())
    }
}
