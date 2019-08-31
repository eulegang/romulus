//! A module which lints a romulus program

use crate::ast::Seq;
use std::fmt;

pub(crate) mod vars;

/// Lints a romulus program with standard linters
///
/// Current linters
/// 1. undefined variables
pub fn lint(node: &Seq) -> Vec<LintMessage> {
    let mut results = Vec::new();

    results.extend(vars::Vars().lint(&node));

    results
}

/// A wrapper type around an error message
#[derive(Clone, PartialEq)]
pub struct LintMessage(String);

impl fmt::Display for LintMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A strategy for linting a romulus program
pub(crate) trait Linter {
    fn lint(&self, node: &Seq) -> Vec<LintMessage>;
}
