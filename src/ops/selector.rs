use super::*;
use crate::node;

pub trait Selector {
    fn select(&self, env: &mut Environment) -> bool;
}

impl Selector for node::Selector {
    fn select(&self, env: &mut Environment) -> bool {
        match self {
            node::Selector::Match(match_node) => match_node.select(env),
            node::Selector::Range(range_node) => range_node.select(env),
        }
    }
}

impl Selector for node::Range {
    fn select(&self, env: &mut Environment) -> bool {
        let node::Range(start, end) = self;

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

impl Selector for node::Match {
    fn select(&self, env: &mut Environment) -> bool {
        match self {
            node::Match::Index(idx) => env.lineno == *idx,
            node::Match::Regex(rgx) => rgx.is_match(&env.line),
        }
    }
}
