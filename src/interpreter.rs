use crate::expression::Expr;
use crate::token::Token;
use crate::token::TokenType;
use crate::value::LoxValue;



pub struct Interpreter {}

impl Interpreter {
    pub fn interpret(&self, expression: Expr) -> Result<LoxValue, String> {
        match expression {
            Expr::Number(number) => Ok(LoxValue::Number(number)),
            Expr::String_(string) => Ok(LoxValue::String(string)),
            Expr::True => Ok(LoxValue::Bool(true)),
            Expr::False => Ok(LoxValue::Bool(false)),
            Expr::Nil => Ok(LoxValue::Nil),
            Expr::Grouping(expression) => self.interpret(*expression),
            Expr::Unary(
                Token {
                    token_type: TokenType::Bang,
                    ..
                },
                expression,
            ) => {
                let boolean = Interpreter::is_truthy(self.interpret(*expression)?);
                Ok(LoxValue::Bool(!boolean))
            }
            Expr::Unary(
                Token {
                    token_type: TokenType::Minus,
                    ..
                },
                expression,
            ) => match self.interpret(*expression)? {
                LoxValue::Number(number) => Ok(LoxValue::Number(-number)),
                _ => Err("Error: operand must be a number.".to_string())
            },
            
            Expr::Binary(left, Token { token_type, line }, right) => {
                match (self.interpret(*left)?, token_type, self.interpret(*right)?) {
                    (LoxValue::Number(left), TokenType::Plus, LoxValue::Number(right)) => {
                        Ok(LoxValue::Number(left + right))
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
                    (LoxValue::String(left), TokenType::Plus, LoxValue::String(right)) => {
                        Ok(LoxValue::String(left + &right))
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
                    (LoxValue::Nil, TokenType::EqualEqual, LoxValue::Nil) => {
                        Ok(LoxValue::Bool(true))
                    }
                    (LoxValue::Nil, TokenType::BangEqual, LoxValue::Nil) => {
                        Ok(LoxValue::Bool(false))
                    }
                    (LoxValue::Number(left), TokenType::EqualEqual, LoxValue::Number(right)) => {
                        Ok(LoxValue::Bool((left - right) < std::f64::EPSILON))
                    }
                    (LoxValue::Number(left), TokenType::BangEqual, LoxValue::Number(right)) => {
                        Ok(LoxValue::Bool(left - right != std::f64::EPSILON))
                    }
                    (LoxValue::String(left), TokenType::EqualEqual, LoxValue::String(right)) => {
                        Ok(LoxValue::Bool(left == right))
                    }
                    (LoxValue::String(left), TokenType::BangEqual, LoxValue::String(right)) => {
                        Ok(LoxValue::Bool(left != right))
                    }
                    (LoxValue::Bool(left), TokenType::EqualEqual, LoxValue::Bool(right)) => {
                        Ok(LoxValue::Bool(left == right))
                    }
                    (LoxValue::Bool(left), TokenType::BangEqual, LoxValue::Bool(right)) => {
                        Ok(LoxValue::Bool(left != right))
                    }
                    (_, TokenType::EqualEqual, _) => Ok(LoxValue::Bool(false)),
                    (_, TokenType::BangEqual, _) => Ok(LoxValue::Bool(true)),
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
                    _ => panic!("Interpreter bug"),
                }
            }
            _ => panic!("Interpreter bug"),
        }
    }

    fn is_truthy(value: LoxValue) -> bool {
        match value {
            LoxValue::Nil => false,
            LoxValue::Number(_) | LoxValue::String(_) => true,
            LoxValue::Bool(boolean) => boolean,
        }
    }
}
