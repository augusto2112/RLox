use std::fmt;

#[derive(Debug, PartialEq, Clone)]
pub enum LoxValue {
    Number(f64),
    String(String),
    Bool(bool),
    Nil,
}

impl fmt::Display for LoxValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            LoxValue::Number(number) => write!(f, "{}", number),
            LoxValue::String(string) => write!(f, "{}", string),
            LoxValue::Bool(boolean) => write!(f, "{}", boolean),
            LoxValue::Nil => write!(f, "nil"),
        }
    }
}

impl LoxValue {
    pub fn is_truthy(&self) -> bool {
        match &self {
            LoxValue::Nil => false,
            LoxValue::Number(_) | LoxValue::String(_) => true,
            LoxValue::Bool(boolean) => *boolean,
        }
    }
}
