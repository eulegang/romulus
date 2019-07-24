
use std::collections::HashMap;
use super::env::Environment;
use regex::Regex;
use std::fmt::{Display, Formatter};
use std::io::Write;

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

pub struct FunctionRegistry {
    funcs: HashMap<String, Box<Func>>
}

impl FunctionRegistry {
    pub fn get(&self, key: &str) -> Option<&Box<Func>> {
        self.funcs.get(key)
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

pub struct Func {
    pub(crate) proc: fn(&mut Environment, args: &[Value]),
}

fn print_impl(env: &mut Environment, args: &[Value]) {
    if args.is_empty() {
        let _ = write!(env.out, "{}", &env.line);
    } else {
        for arg in args {
            let _ = write!(env, "{}", arg);
        }
    }

    let _ = write!(env, "\n");
}
