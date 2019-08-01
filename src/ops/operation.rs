use super::*;
use crate::runtime::Value;
use crate::ast;

pub trait Operation {
    fn perform(&self, env: &mut Environment);
}

impl Operation for ast::Seq {
    fn perform(&self, env: &mut Environment) {
        for sub in &self.subnodes {
            sub.perform(env)
        }
    }
}

impl Operation for ast::Body {
    fn perform(&self, env: &mut Environment) {
        match self {
            ast::Body::Bare(func_node) => func_node.perform(env),
            ast::Body::Guard(sel_node, node) => {
                if sel_node.select(env) {
                    env.push(sel_node.scope(env));
                    node.perform(env);
                    env.pop();
                }
            }
        }
    }
}

impl Operation for ast::Function {
    fn perform(&self, env: &mut Environment) {
        let values: Vec<Value> = self
            .args
            .iter()
            .map(|expr| -> Value { expr.to_value(&env) })
            .collect();

        env.call(self.name.clone(), values);
    }
}
