use crate::lang::func::Value;
use super::*;

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
                if *interpolate {
                    Value::String(s.clone()) // TODO: Don't interpolate for the time being
                } else {
                    Value::String(s.clone())
                }
            }
        }
    }
}

