use super::Scope;

pub(crate) struct RangeScopeTracker {
    states: Vec<Option<Scope>>,
    pos: usize,
}

impl RangeScopeTracker {
    pub(crate) fn new(cap: usize) -> RangeScopeTracker {
        RangeScopeTracker {
            states: vec![None; cap],
            pos: 0,
        }
    }

    pub(crate) fn in_range(&self) -> bool {
        self.states[self.pos].is_some()
    }

    pub(crate) fn next(&mut self) {
        self.pos = (self.pos + 1) % self.states.len();
    }

    pub(crate) fn get(&self) -> &Option<Scope> {
        &self.states[self.pos]
    }

    pub(crate) fn set(&mut self, scope: Scope) {
        self.states[self.pos] = Some(scope);
    }

    pub(crate) fn clear(&mut self) {
        self.states[self.pos] = None
    }
}
