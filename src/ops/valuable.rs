use super::*;
use regex::{Captures, Regex};
use crate::node::*;
use crate::runtime::Value;

pub trait Valuable {
    fn to_value(&self, env: &Environment) -> Value;
}

impl Valuable for Expression {
    fn to_value(&self, env: &Environment) -> Value {
        match self {
            Expression::Literal(lit) => lit.to_value(env),

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
            Literal::String(s, interpolate) => {
                lazy_static! {
                    static ref INTERPOLATOR: Regex =
                        Regex::new(r"\$\{(?P<name>[a-zA-Z0-9_]+)\}").unwrap();
                }

                if *interpolate {
                    Value::String(
                        INTERPOLATOR
                            .replace_all(s, |capture: &Captures| -> String {
                                let key = String::from(&capture["name"]);
                                env.lookup(&key).unwrap_or_default()
                            })
                            .to_owned()
                            .to_string(),
                    )
                } else {
                    Value::String(s.clone())
                }
            }
        }
    }
}
