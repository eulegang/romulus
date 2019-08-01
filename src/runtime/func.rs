use crate::runtime::Environment;
use regex::Regex;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use super::func_impl;

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

/// A registry containing function implemenations
pub struct FunctionRegistry {
    funcs: HashMap<String, Func>,
}

impl FunctionRegistry {
    /// get a function from the registry
    pub fn get(&self, key: &str) -> Option<&Func> {
        self.funcs.get(key)
    }

    /// add a function to the registry
    /// 
    /// functions may not be overriden in the registry
    /// if a name is already registred this method will
    /// return `false`
    pub fn put(&mut self, key: &str, func: Func) -> bool {
        if self.funcs.get(key).is_some() {
            false
        } else {
            self.funcs.insert(key.to_string(), func);
            true
        }
    }
}

impl Default for FunctionRegistry {
    fn default() -> FunctionRegistry {
        let mut funcs = HashMap::new();

        funcs.insert(String::from("print"), Func { proc: func_impl::print });

        FunctionRegistry { funcs }
    }
}

/// A wrapper for function implementations
pub struct Func {
    pub proc: fn(env: &mut Environment, args: &[Value]),
}
