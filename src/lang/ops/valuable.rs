use super::*;
use crate::lang::func::Value;
use regex::{Captures, Regex};

pub trait Valuable {
    fn to_value(&self, env: &Environment) -> Value;
}

impl Valuable for ExpressionNode {
    fn to_value(&self, env: &Environment) -> Value {
        match self {
            ExpressionNode::Literal(lit) => lit.to_value(env),

            ExpressionNode::Identifier(name) => {
                if let Some(value) = env.lookup(name) {
                    Value::String(value.clone())
                } else {
                    Value::String(String::new())
                }
            }
        }
    }
}

impl Valuable for LiteralNode {
    fn to_value(&self, env: &Environment) -> Value {
        match self {
            LiteralNode::Regex(regex) => Value::Regex(regex.clone()),
            LiteralNode::String(s, interpolate) => {
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
