use super::{Lint, LintMessage};
use crate::ast::*;
use crate::ops::interpolated_variables;

pub(super) struct Vars();

impl Vars {
    fn lint_vars(&self, node: &Seq, vars: &mut Vec<Vec<String>>) -> Vec<String> {
        let mut results = Vec::new();
        for node in &node.subnodes {
            match node {
                Body::Bare(stmt) => {
                    results.extend(self.lint_check(vars, stmt.consumes()));
                }

                Body::Single(sel, stmt) => {
                    vars.push(sel.provides());

                    results.extend(self.lint_check(vars, stmt.consumes()));

                    vars.pop();
                }

                Body::Guard(sel, seq) => {
                    vars.push(sel.provides());

                    self.lint_vars(seq, vars);

                    vars.pop();
                }
            }
        }
        results
    }

    fn lint_check(&self, vars: &Vec<Vec<String>>, needed: Vec<String>) -> Vec<String> {
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
}

impl Lint for Vars {
    fn lint(&self, node: &Seq) -> Vec<LintMessage> {
        let mut vars = Vec::new();

        vars.push(vec!["_".to_string()]);

        if cfg!(feature = "envvar") {
            vars.push(env_vars());
        }

        let mut violations = self.lint_vars(node, &mut vars);
        violations.dedup();

        violations
            .iter()
            .map(format_message)
            .collect()
    }
}

trait ScopeProvider {
    fn provides(&self) -> Vec<String>;
}

impl ScopeProvider for Selector {
    fn provides(&self) -> Vec<String> {
        match self {
            Selector::Match(m) => m.provides(),
            Selector::Range(r) => r.provides(),
            Selector::Pattern(p) => p.provides(),
        }
    }
}

impl ScopeProvider for Match {
    fn provides(&self) -> Vec<String> {
        let mut results = Vec::new();

        if let Match::Regex(regex) = self {
            for o in regex.capture_names() {
                if let Some(name) = o {
                    results.push(name.to_string());
                }
            }
        }

        results
    }
}

impl ScopeProvider for Range {
    fn provides(&self) -> Vec<String> {
        self.0.provides()
    }
}

impl ScopeProvider for PatternMatch {
    fn provides(&self) -> Vec<String> {
        let mut result = Vec::new();
        for pat in &self.patterns {
            result.extend(pat.provides());
        }
        result
    }
}

impl ScopeProvider for Pattern {
    fn provides(&self) -> Vec<String> {
        match self {
            Pattern::String(_, _) => Vec::new(),
            Pattern::Identifier(s) => vec![s.clone()],
            Pattern::Regex(regex) => {
                let mut results = Vec::new();
                for o in regex.capture_names() {
                    if let Some(name) = o {
                        results.push(name.to_string());
                    }
                }

                results
            }
        }
    }
}

trait ScopeConsumer {
    fn consumes(&self) -> Vec<String>;
}

impl ScopeConsumer for Statement {
    fn consumes(&self) -> Vec<String> {
        use Statement::*;

        match self {
            Print(expr) => expr.consumes(),
            Quit => vec![],
            Subst(_, expr) => expr.consumes(),
            Gsubst(_, expr) => expr.consumes(),
            Read(expr) => expr.consumes(),
            Write(expr) => expr.consumes(),
            Exec(expr) => expr.consumes(),
        }
    }
}

impl ScopeConsumer for Expression {
    fn consumes(&self) -> Vec<String> {
        use Expression::*;
        match self {
            Identifier(name) => vec![name.to_string()],
            String(_, false) => vec![],

            String(content, true) => {
                interpolated_variables(content)
            }
        }
    }
}

fn env_vars() -> Vec<String> {
    let mut vars = Vec::new();
    for (key, _) in std::env::vars() {
        vars.push(key)
    }
    vars
}

#[cfg(feature = "color")]
fn format_message(variable: &String) -> LintMessage {
    LintMessage(format!("Undefined variable {}", ansi_term::Style::new().bold().paint(variable)))
}

#[cfg(not(feature = "color"))]
fn format_message(variable: &String) -> LintMessage {
    LintMessage(format!("Undefined variable {}", variable))
}
