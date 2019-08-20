//! A module which lints a romulus program

use crate::ast::Seq;
use std::fmt;

mod vars;

/// A wrapper type around an error message
#[derive(Clone, PartialEq)]
pub struct LintMessage(String);

pub(crate) trait Lint {
    fn lint(&self, node: &Seq) -> Vec<LintMessage>;
}

/// Lints a romulus program with standard linters
///
/// Current linters 
/// 1. undefined variables
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

