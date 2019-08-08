use regex::Regex;
use std::fmt::{Display, Formatter};

/// A value used at runtime
pub enum Value {
    String(String),
    Regex(Box<Regex>),
}

impl Display for Value {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Value::String(s) => write!(fmt, "{}", s),
            Value::Regex(r) => write!(fmt, "{}", r),
        }
    }
}
