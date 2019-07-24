use super::*;

pub trait ScopeProvider {
    fn scope(&self, env: &Environment) -> Scope;
}

impl ScopeProvider for SelectorNode {
    fn scope(&self, env: &Environment) -> Scope {
        match self {
            SelectorNode::Match(match_node) => match_node.scope(env),
            SelectorNode::Range(range_node) => range_node.scope(env),
        }
    }
}

impl ScopeProvider for RangeNode {
    fn scope(&self, env: &Environment) -> Scope {
        if let Some(scope) = env.range_scope() {
            scope.clone()
        } else {
            Scope::new()
        }
    }
}

impl ScopeProvider for MatchNode {
    fn scope(&self, env: &Environment) -> Scope {
        let mut scope = Scope::new();

        match self {
            MatchNode::Index(_) => (),
            MatchNode::Regex(rgx) => {
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


