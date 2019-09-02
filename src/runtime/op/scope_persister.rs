use super::*;
use crate::ast;

pub trait ScopePersister {
    fn persist_scope(&self, env: &mut Environment);
}

impl ScopePersister for ast::Selector {
    fn persist_scope(&self, env: &mut Environment) {
        use ast::Selector::*;

        match self {
            Range(r) => r.persist_scope(env),
            Negate(s) => s.persist_scope(env),
            Disjunction(lh, rh) => {
                lh.persist_scope(env);
                rh.persist_scope(env);
            }

            Conjunction(lh, rh) => {
                lh.persist_scope(env);
                rh.persist_scope(env);
            }

            _ => (),
        }
    }
}

impl ScopePersister for ast::Seq {
    fn persist_scope(&self, env: &mut Environment) {
        for node in &self.subnodes {
            node.persist_scope(env);
        }
    }
}

impl ScopePersister for ast::Body {
    fn persist_scope(&self, env: &mut Environment) {
        use ast::Body::*;
        match self {
            Bare(_) => (),
            Single(s, _) => s.persist_scope(env),
            Guard(s, _) => s.persist_scope(env),
        }
    }
}

impl ScopePersister for ast::Range {
    fn persist_scope(&self, env: &mut Environment) {
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
    }
}
