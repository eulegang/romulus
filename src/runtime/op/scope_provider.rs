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
            ast::Selector::Pattern(pattern_match_node) => pattern_match_node.scope(env),
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
            ast::Match::Begin => (),
            ast::Match::End => (),
            ast::Match::Index(_) => (),
            ast::Match::Regex(rgx) => {
                if let Event::Line(line) = &env.event {
                    scope += Scope::from_regex(rgx, line)
                }
            }
        }

        scope
    }
}

impl ScopeProvider for ast::PatternMatch {
    fn scope(&self, env: &Environment) -> Scope {
        env.split_line(|parts| {
            let mut scope = Scope::new();

            for pattern in &self.patterns {
                let part = match parts.next() {
                    Some(part) => part,
                    None => return Scope::new(),
                };

                match pattern {
                    ast::Pattern::Identifier(id) if id == "_" => (),
                    ast::Pattern::Identifier(id) => scope.set(id.clone(), part.to_string()),
                    ast::Pattern::String(_, _) => (),
                    ast::Pattern::Regex(rgx) => scope += Scope::from_regex(rgx, part),
                }
            }

            scope
        })
        .unwrap_or(Scope::new())
    }
}
