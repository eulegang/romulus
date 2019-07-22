
use std::collections::HashMap;
use super::env::Environment;
use regex::Regex;
use std::fmt::{Display, Formatter};
use std::io::Write;

pub(crate) enum FnCallError {
    UnknownFunction(String)
}

pub(crate) enum Value {
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

pub struct FunctionRegistry {
    funcs: HashMap<String, Box<Func>>
}

impl FunctionRegistry {
    pub(crate) fn call(&self, name: String, values: Vec<Value>) -> Result<(), FnCallError> {
        unimplemented!();
    }
}

impl Default for FunctionRegistry {
    fn default() -> FunctionRegistry {
        let mut funcs = HashMap::new();

        funcs.insert(String::from("print"), Box::new(Func{ proc: print_impl }));

        FunctionRegistry {
            funcs: funcs,
        }
    }
}

struct Func {
    proc: fn(&mut Environment, args: &[Value]),
}

fn print_impl(env: &mut Environment, args: &[Value]) {
    for arg in args {
        let _ = write!(env, "{}", arg);
    }

    let _ = write!(env, "\n");
}
