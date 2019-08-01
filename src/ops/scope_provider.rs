use super::*;
use crate::node;

pub trait ScopeProvider {
    fn scope(&self, env: &Environment) -> Scope;
}

impl ScopeProvider for node::Selector {
    fn scope(&self, env: &Environment) -> Scope {
        match self {
            node::Selector::Match(match_node) => match_node.scope(env),
            node::Selector::Range(range_node) => range_node.scope(env),
        }
    }
}

impl ScopeProvider for node::Range {
    fn scope(&self, env: &Environment) -> Scope {
        if let Some(scope) = env.tracker.get() {
            scope.clone()
        } else {
            Scope::new()
        }
    }
}

impl ScopeProvider for node::Match {
    fn scope(&self, env: &Environment) -> Scope {
        let mut scope = Scope::new();

        match self {
            node::Match::Index(_) => (),
            node::Match::Regex(rgx) => {
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
