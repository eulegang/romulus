use super::*;
use crate::func::Value;
use crate::node;

pub trait Operation {
    fn perform(&self, env: &mut Environment);
}

impl Operation for node::Seq {
    fn perform(&self, env: &mut Environment) {
        for sub in &self.subnodes {
            sub.perform(env)
        }
    }
}

impl Operation for node::Body {
    fn perform(&self, env: &mut Environment) {
        match self {
            node::Body::Bare(func_node) => func_node.perform(env),
            node::Body::Guard(sel_node, node) => {
                if sel_node.select(env) {
                    env.push(sel_node.scope(env));
                    node.perform(env);
                    env.pop();
                }
            }
        }
    }
}

impl Operation for node::Function {
    fn perform(&self, env: &mut Environment) {
        let values: Vec<Value> = self
            .args
            .iter()
            .map(|expr| -> Value { expr.to_value(&env) })
            .collect();

        env.call(self.name.clone(), values);
    }
}
