use crate::token::Token;
use crate::token::TokenType;
use crate::value::LoxValue;
use std::collections::HashMap;
use std::mem::discriminant;


#[derive(Debug, Clone)]
pub struct Environment {
    enclosed: Option<Box<Environment>>,
    values: HashMap<String, LoxValue>,
}

impl Environment {
    pub fn new() -> Environment {
        Environment {
            enclosed: Option::None,
            values: HashMap::new(),
        }
    }

    pub fn add_subenvironment(&mut self) {
        if let Some(ref mut enclosed) = self.enclosed {
            enclosed.add_subenvironment()
        } else {
            self.enclosed = Option::from(Box::from(Environment::new()))
        }
    }

    fn contains_subenvironmnet(&self) -> bool {
        discriminant(&self.enclosed) != discriminant(&Option::None)
    }

    pub fn remove_subenvironment(&mut self) {
        if let Some(ref mut enclosed) = self.enclosed {
            if enclosed.contains_subenvironmnet() {
                enclosed.remove_subenvironment();
                return 
            }
        }
        self.enclosed = Option::None
    }

    pub fn define(&mut self, name: &str, value: &LoxValue) {
        if let Some(ref mut enclosed) = self.enclosed {
            enclosed.define(name, value)
        } else {
            self.values.insert(name.to_string(), value.clone());
        }
    }

    pub fn assign(&mut self, name: &str, value: &LoxValue) -> Result<(), String> {
        if let Some(ref mut enclosed) = self.enclosed {
            let result = enclosed.assign(name, value);
            if result.is_ok() {
                return result
            }
        }       

        if self.values.contains_key(name) {
            self.values.insert(name.to_string(), value.clone());
            return Ok(())
        }

        Err(format!("Undefined variable '{}'.", name))        
    }


    pub fn get(&self, token: &Token) -> Result<LoxValue, String> {
        if let Some(enclosed) = &self.enclosed {
            let result = enclosed.get(token);
            if result.is_ok() {
                return result
            }
        }

        if let TokenType::Identifier(identifier) = &token.token_type {
            self.values
                .get(identifier)
                .ok_or({ format!("Undefined variable: {}", identifier) })
                .map(|value| value.clone())
        } else if let Some(enclosed) = &self.enclosed {
            enclosed.get(token)
        }else {
            panic!("Compiler bug: unexpected token: {:?}", token);
        }
    }
}
