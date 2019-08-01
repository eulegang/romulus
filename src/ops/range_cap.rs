use crate::node;

//
// Specifies how many ranges need to be kept track of
//
pub trait RangeCap {
    fn num_ranges(&self) -> usize;
}

impl RangeCap for node::Seq {
    fn num_ranges(&self) -> usize {
        let mut count = 0;
        for sub in &self.subnodes {
            count += sub.num_ranges();
        }

        count
    }
}

impl RangeCap for node::Body {
    fn num_ranges(&self) -> usize {
        match self {
            node::Body::Bare(_) => 0,
            node::Body::Guard(sel, node) => sel.num_ranges() + node.num_ranges(),
        }
    }
}

impl RangeCap for node::Selector {
    fn num_ranges(&self) -> usize {
        match self {
            node::Selector::Match(_) => 0,
            node::Selector::Range(_) => 1,
        }
    }
}
