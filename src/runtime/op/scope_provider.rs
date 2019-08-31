use super::*;
use crate::ast;

use crate::lint::vars::provider::ScopeProvider as VarProvider;

pub trait ScopeProvider {
    fn scope(&self, env: &Environment) -> Scope;
}

impl ScopeProvider for ast::Selector {
    fn scope(&self, env: &Environment) -> Scope {
        use ast::Selector::*;
        match self {
            Match(match_node) => match_node.scope(env),
            Range(range_node) => range_node.scope(env),
            Pattern(pattern_match_node) => pattern_match_node.scope(env),
            Negate(_) => Scope::default(),
            Conjunction(lh, rh) => lh.scope(env) + rh.scope(env),
            Disjunction(lh, rh) => (lh.scope(env) + rh.scope(env)).pick(&self.provides()),
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
        .unwrap_or_default()
    }
}
