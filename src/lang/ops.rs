use super::nodes::*;
use super::env::{Environment,Scope};

pub trait Operation {
    fn perform(&self, env: &mut Environment);
}

trait Selector {
    fn select(&self, env: &mut Environment) -> bool;
}

trait ScopeProvider {
    fn scope(&self, env: &Environment) -> Scope;
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

impl Selector for SelectorNode {
    fn select(&self, env: &mut Environment) -> bool {
        match self {
            SelectorNode::Match(match_node) => match_node.select(env),
            SelectorNode::Range(range_node) => range_node.select(env),
        }
    }
}

impl ScopeProvider for SelectorNode {
    fn scope(&self, env: &Environment) -> Scope {
        match self {
            SelectorNode::Match(match_node) => match_node.scope(env),
            SelectorNode::Range(range_node) => range_node.scope(env),
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

impl ScopeProvider for RangeNode {
    fn scope(&self, env: &Environment) -> Scope {
        unimplemented!();
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

impl ScopeProvider for MatchNode {
    fn scope(&self, env: &Environment) -> Scope {
        let mut scope = Scope::new();

        match self {
            MatchNode::Index(_) => (),
            MatchNode::Regex(rgx) => {
                if let Some(capture) = rgx.captures(&env.line) {
                    for name in rgx.capture_names() {
                        if let Some(n) = name {
                            if let Some(m) = capture.name(n) {
                                scope.set(n.to_string(), m.as_str().to_string())
                            }
                        }
                    }
                }
            }
        }

        scope
    }
}


