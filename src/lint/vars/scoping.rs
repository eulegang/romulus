use super::consumer::ScopeConsumer;
use super::provider::ScopeProvider;
use crate::ast::Body::*;
use crate::ast::Seq;

pub(super) fn lint_vars(node: &Seq, vars: &mut Vec<Vec<String>>) -> Vec<String> {
    let mut results = Vec::new();
    for node in &node.subnodes {
        match node {
            Bare(stmt) => {
                results.extend(check_vars(vars, stmt.consumes()));
            }

            Single(sel, stmt) => {
                vars.push(sel.provides());

                results.extend(check_vars(vars, stmt.consumes()));

                vars.pop();
            }

            Guard(sel, seq) => {
                vars.push(sel.provides());

                results.extend(lint_vars(seq, vars));

                vars.pop();
            }
        }
    }
    results
}

fn check_vars(vars: &[Vec<String>], needed: Vec<String>) -> Vec<String> {
    let mut violations = Vec::new();

    for consumed in needed {
        let mut resolved = false;
        for provided in vars {
            if provided.contains(&consumed) {
                resolved = true;
                break;
            }
        }

        if !resolved {
            violations.push(consumed)
        }
    }

    violations
}
