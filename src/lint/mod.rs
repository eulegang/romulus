
use crate::ast::Seq;
use std::fmt;

mod vars;

#[derive(Clone, PartialEq)]
pub struct LintMessage(String);

pub trait Lint {
    fn lint(&self, node: &Seq) -> Vec<LintMessage>;
}

pub fn lint(node: &Seq) -> Vec<LintMessage> {
    let mut results = Vec::new();

    results.extend(vars::Vars().lint(&node));
    
    results
}

impl fmt::Display for LintMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

