use super::*;
use crate::nodes::*;

pub trait Selector {
    fn select(&self, env: &mut Environment) -> bool;
}

impl Selector for SelectorNode {
    fn select(&self, env: &mut Environment) -> bool {
        match self {
            SelectorNode::Match(match_node) => match_node.select(env),
            SelectorNode::Range(range_node) => range_node.select(env),
        }
    }
}

impl Selector for RangeNode {
    fn select(&self, env: &mut Environment) -> bool {
        let RangeNode(start, end) = self;

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

impl Selector for MatchNode {
    fn select(&self, env: &mut Environment) -> bool {
        match self {
            MatchNode::Index(idx) => env.lineno == *idx,
            MatchNode::Regex(rgx) => rgx.is_match(&env.line),
        }
    }
}
