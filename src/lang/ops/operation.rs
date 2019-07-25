use super::*;
use crate::lang::func::Value;

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
        let values: Vec<Value> = self
            .args
            .iter()
            .map(|expr| -> Value { expr.to_value(&env) })
            .collect();

        env.call(self.name.clone(), values);
    }
}
