use super::*;

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

        if !env.range() {
            if start.select(env) {
                env.set_range_state(start.scope(env));
            }
        } else {
            if end.select(env) {
                env.clear_range_state();
            }
        };

        let next = env.range();
        env.next_range();
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
