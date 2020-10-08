use crate::ast::{Body, Match, Selector};
use crate::runtime::Event;

pub(crate) trait Lifecycle {
    fn is_lifecycle(&self) -> bool;
}

impl Lifecycle for Event {
    fn is_lifecycle(&self) -> bool {
        matches!(self, Event::Begin | Event::End)
    }
}

impl Lifecycle for Body {
    fn is_lifecycle(&self) -> bool {
        match self {
            Body::Bare(_) => false,
            Body::Single(sel, _) => sel.is_lifecycle(),
            Body::Guard(sel, _) => sel.is_lifecycle(),
        }
    }
}

impl Lifecycle for Selector {
    fn is_lifecycle(&self) -> bool {
        match self {
            Selector::Match(m) => m.is_lifecycle(),
            _ => false,
        }
    }
}

impl Lifecycle for Match {
    fn is_lifecycle(&self) -> bool {
        matches!(self, Match::Begin | Match::End)
    }
}
