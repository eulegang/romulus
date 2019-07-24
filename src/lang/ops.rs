use super::nodes::*;
use super::env::Environment;

pub trait Operation {
    fn perform(&self, env: &mut Environment);
}

trait Selector {
    fn select(&self, env: &mut Environment) -> bool;
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
                    node.perform(env);
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
    fn select(&self, env: &mut Environment) -> bool {
        match self {
            SelectorNode::Match(match_node) => match_node.select(env),
            SelectorNode::Range(range_node) => range_node.select(env),
        }
    }
}

impl Selector for RangeNode {
    fn select(&self, env: &mut Environment) -> bool {
        let RangeNode(start, end) = self;

        let next = if !env.range() {
            if start.select(env) {
                env.toggle_range();
            }
            env.range()
        } else {
            if end.select(env) {
                env.toggle_range();
                !env.range()
            } else {
                env.range()
            }
        };

        env.next_range();
        next
    }
}

impl Selector for MatchNode {
    fn select(&self, env: &mut Environment) -> bool {
        match self {
            MatchNode::Index(idx) => env.lineno == *idx,
            MatchNode::Regex(rgx) => rgx.is_match(&env.line),
        }
    }
}

