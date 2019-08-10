use super::*;
use regex::{Captures, Regex};
use crate::ast::*;
use crate::runtime::Value;

pub trait Valuable {
    fn to_value(&self, env: &Environment) -> Value;
}

lazy_static! {
    static ref INTERPOLATOR: Regex =
        Regex::new(r"\$\{(?P<name>[a-zA-Z0-9_]+)\}").unwrap();
}

impl Valuable for Expression {
    fn to_value(&self, env: &Environment) -> Value {
        match self {
            Expression::String(content, interpolatable) =>  {
                if *interpolatable {
                    interpolate(content, env)
                } else {
                    Value::String(content.clone())
                }
            }

            Expression::Identifier(name) => {
                if let Some(value) = env.lookup(name) {
                    Value::String(value.clone())
                } else {
                    Value::String(String::new())
                }
            }
        }
    }
}

impl Valuable for Literal {
    fn to_value(&self, env: &Environment) -> Value {
        match self {
            Literal::Regex(regex) => Value::Regex(regex.clone()),
            Literal::String(s, interpolatable) => {
                if *interpolatable {
                    interpolate(s, env)
                } else {
                    Value::String(s.clone())
                }
            }
        }
    }
}


fn interpolate(content: &str, env: &Environment) -> Value {
    let intermediary = content.replace("\\$", "\0");
    let eval = |capture: &Captures| -> String {
        let key = String::from(&capture["name"]);
        env.lookup(&key).unwrap_or_default()
    };

    let evaled = INTERPOLATOR.replace_all(&intermediary, eval).to_owned().to_string();

    Value::String(evaled.replace('\0', "$"))
}
