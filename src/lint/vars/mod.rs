use super::{Linter, LintMessage};
use crate::ast::*;

mod consumer;
mod provider;
mod scoping;

use consumer::{env_vars};
use scoping::*;

pub(super) struct Vars();

impl Linter for Vars {
    fn lint(&self, node: &Seq) -> Vec<LintMessage> {
        let mut vars = Vec::new();

        vars.push(vec!["_".to_string()]);

        if cfg!(feature = "envvar") {
            vars.push(env_vars());
        }

        let mut violations = lint_vars(node, &mut vars);
        violations.dedup();

        violations
            .iter()
            .map(|var| LintMessage(format!("Undefined variable {}", color!(ansi_term::Style::new().bold(), var))))
            .collect()
    }
}

