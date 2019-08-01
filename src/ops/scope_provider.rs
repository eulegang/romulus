use super::*;
use crate::ast;

pub trait ScopeProvider {
    fn scope(&self, env: &Environment) -> Scope;
}

impl ScopeProvider for ast::Selector {
    fn scope(&self, env: &Environment) -> Scope {
        match self {
            ast::Selector::Match(match_node) => match_node.scope(env),
            ast::Selector::Range(range_node) => range_node.scope(env),
        }
    }
}

impl ScopeProvider for ast::Range {
    fn scope(&self, env: &Environment) -> Scope {
        if let Some(scope) = env.tracker.get() {
            scope.clone()
        } else {
            Scope::new()
        }
    }
}

impl ScopeProvider for ast::Match {
    fn scope(&self, env: &Environment) -> Scope {
        let mut scope = Scope::new();

        match self {
            ast::Match::Index(_) => (),
            ast::Match::Regex(rgx) => {
                if let Some(capture) = rgx.captures(&env.line) {
                    for name in rgx.capture_names() {
                        if let Some(n) = name {
                            if let Some(m) = capture.name(n) {
                                scope.set(n.to_string(), m.as_str().to_string())
                            }
                        }
                    }
                }
            }
        }

        scope
    }
}
