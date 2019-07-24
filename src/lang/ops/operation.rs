use crate::lang::func::Value;
use super::*;

pub trait Operation {
    fn perform(&self, env: &mut Environment);
}

impl Operation for Node {
    fn perform(&self, env: &mut Environment) {
        for sub in &self.subnodes {
            sub.perform(env)
        }
    }
}

impl Operation for BodyNode {
    fn perform(&self, env: &mut Environment) {
        match self {
            BodyNode::Bare(func_node) => func_node.perform(env),
            BodyNode::Guard(sel_node, node) => {
                if sel_node.select(env) {
                    env.push(sel_node.scope(env));
                    node.perform(env);
                    env.pop();
                }
            }
        }
    }
}

impl Operation for FunctionNode {
    fn perform(&self, env: &mut Environment) {
        let values: Vec<Value> = self.args.iter().map(|expr| -> Value {
            match expr {
                ExpressionNode::Literal(lit) => {
                    match lit {
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

                ExpressionNode::Identifier(name) => {
                    if let Some(value) = env.lookup(name) {
                        Value::String(value.clone())
                    } else {
                        Value::String(String::new())
                    }
                }
            }
        }).collect();

        env.call(self.name.clone(), values);
    }
}




