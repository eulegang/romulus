use super::nodes::*;
use super::env::Environment;

pub trait Operation {
    fn perform(&self, env: &mut Environment);
}

trait Selector {
    fn select(&self, env: &Environment) -> bool;
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
            BodyNode::Guard(sel_node, expr_node) => {
                if sel_node.select(env) {
                    expr_node.perform(env);
                }
            }
        }
    }
}

use super::func::Value;

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
            }
        }).collect();

        env.call(self.name.clone(), values);
    }
}

impl Selector for SelectorNode {
    fn select(&self, env: &Environment) -> bool {
        match self {
            SelectorNode::Match(match_node) => match_node.select(env),
            SelectorNode::Range(range_node) => range_node.select(env),
        }
    }
}

impl Selector for RangeNode {
    fn select(&self, env: &Environment) -> bool {
        unimplemented!()
    }
}

impl Selector for MatchNode {
    fn select(&self, env: &Environment) -> bool {
        match self {
            MatchNode::Index(idx) => env.lineno == *idx,
            MatchNode::Regex(rgx) => rgx.is_match(&env.line),
        }
    }
}

