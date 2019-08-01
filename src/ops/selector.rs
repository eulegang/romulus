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
            ast::Match::Index(idx) => env.lineno == *idx,
            ast::Match::Regex(rgx) => rgx.is_match(&env.line),
        }
    }
}
