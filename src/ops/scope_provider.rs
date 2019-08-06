use super::*;
use crate::ast;
use regex::Regex;

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
                    extract_regex_scope(&mut scope, rgx, line)
                }
            }
        }

        scope
    }
}

impl ScopeProvider for ast::PatternMatch {
    fn scope(&self, env: &Environment) -> Scope {
        let mut scope = Scope::new();
        
        let line = match &env.event {
            Event::Line(line) => line,
            _ => return scope,
        };

        
        let mut parts = env.seperator.split(&line);
        for pattern in &self.patterns {
            let part = match parts.next() {
                Some(part) => part,
                None => return Scope::new(),
            };

            match pattern {
                ast::Pattern::Identifier(id) if id == "_" => (),
                ast::Pattern::Identifier(id) => scope.set(id.clone(), part.to_string()),
                ast::Pattern::Literal(ast::Literal::String(_, _)) => (),
                ast::Pattern::Literal(ast::Literal::Regex(rgx)) => extract_regex_scope(&mut scope, rgx, line)
            }
        }

        scope
    }
}

#[inline]
fn extract_regex_scope(scope: &mut Scope, regex: &Regex, line: &str) {
    if let Some(capture) = regex.captures(line) {
        for name in regex.capture_names() {
            if let Some(n) = name {
                if let Some(m) = capture.name(n) {
                    scope.set(n.to_string(), m.as_str().to_string())
                }
            }
        }
    }
}
