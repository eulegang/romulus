
use crate::ast::Seq;

mod vars;

pub trait Lint {
    fn lint(&self, node: &Seq) -> Vec<String>;
}

pub fn lint(node: &Seq) -> Vec<String> {
    let mut results = Vec::new();

    results.extend(vars::Vars().lint(&node));
    
    results
}

