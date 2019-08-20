use super::*;
use crate::ast;

pub trait Selector {
    fn select(&self, env: &mut Environment) -> bool;
}

impl Selector for ast::Selector {
    fn select(&self, env: &mut Environment) -> bool {
        match self {
            ast::Selector::Match(match_node) => match_node.select(env),
            ast::Selector::Range(range_node) => range_node.select(env),
            ast::Selector::Pattern(pattern_node) => pattern_node.select(env),
        }
    }
}

impl Selector for ast::Range {
    fn select(&self, env: &mut Environment) -> bool {
        let ast::Range(start, end) = self;

        if !env.tracker.in_range() {
            if start.select(env) {
                env.tracker.set(start.scope(env));
            }
        } else if end.select(env) {
            env.tracker.clear();

            if start.select(env) {
                env.tracker.set(start.scope(env));
            }
        };

        let next = env.tracker.in_range();
        env.tracker.next();
        next
    }
}

impl Selector for ast::Match {
    fn select(&self, env: &mut Environment) -> bool {
        match self {
            ast::Match::Begin => env.event == Event::Begin,
            ast::Match::End => env.event == Event::End,
            ast::Match::Index(idx) => env.lineno == *idx,
            ast::Match::Regex(rgx) => {
                if let Event::Line(line) = &env.event {
                    rgx.is_match(line)
                } else {
                    false
                }
            }
        }
    }
}

impl Selector for ast::PatternMatch {
    fn select(&self, env: &mut Environment) -> bool {
        let line = match &env.event {
            Event::Line(line) => line,
            _ => return false,
        };

        let mut parts = env.seperator.split(&line);
        for pattern in &self.patterns {
            let part = match parts.next() {
                Some(part) => part,
                None => return false,
            };

            match pattern {
                ast::Pattern::Identifier(_) => continue,
                ast::Pattern::Regex(regex) => {
                    if !regex.is_match(part) {
                        return false;
                    }
                }

                ast::Pattern::String(s, false) => {
                    if s != part {
                        return false;
                    }
                }

                ast::Pattern::String(s, true) => {
                    if interpolate(&s, env) != part {
                        return false;
                    }
                }
            };
        }

        true
    }
}
