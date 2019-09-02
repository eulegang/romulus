use super::*;
use crate::ast;

pub trait Selector {
    fn select(&self, env: &mut Environment) -> bool;
}

impl Selector for ast::Selector {
    fn select(&self, env: &mut Environment) -> bool {
        use ast::Selector::*;
        match self {
            Match(match_node) => match_node.select(env),
            Range(range_node) => range_node.select(env),
            Pattern(pattern_node) => pattern_node.select(env),
            Negate(selector) => !selector.select(env),
            Conjunction(lh, rh) => {
                // Eagerly evauting since ranges are stateful
                let l = lh.select(env);
                let r = rh.select(env);

                l && r
            }
            Disjunction(lh, rh) => {
                // Eagerly evauting since ranges are stateful
                let l = lh.select(env);
                let r = rh.select(env);

                l || r
            }
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
        use ast::Match::*;

        match self {
            Begin => env.event == Event::Begin,
            End => env.event == Event::End,
            Index(idx) => env.lineno == *idx,
            Regex(rgx) => {
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
        use ast::Pattern::*;

        env.split_line(|parts| {
            for pattern in &self.patterns {
                let part = match parts.next() {
                    Some(part) => part,
                    None => return false,
                };

                match pattern {
                    Regex(regex) if !regex.is_match(part) => return false,
                    String(s, false) if s != part => return false,
                    String(s, true) if interpolate(&s, env) != part => return false,

                    Identifier(_) => continue,
                    _ => continue,
                };
            }

            true
        })
        .unwrap_or(false)
    }
}
