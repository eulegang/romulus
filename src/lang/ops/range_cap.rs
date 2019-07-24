use super::*;


//
// Specifies how many ranges need to be kept track of
//
pub trait RangeCap {
    fn num_ranges(&self) -> usize;
}

impl RangeCap for Node {
    fn num_ranges(&self) -> usize {
        let mut count = 0;
        for sub in &self.subnodes {
            count += sub.num_ranges();
        }

        count
    }
}

impl RangeCap for BodyNode {
    fn num_ranges(&self) -> usize {
        match self {
            BodyNode::Bare(_) => 0,
            BodyNode::Guard(sel, node) => sel.num_ranges() + node.num_ranges(),
        }
    }
}

impl RangeCap for SelectorNode {
    fn num_ranges(&self) -> usize {
        match self {
            SelectorNode::Match(_) => 0,
            SelectorNode::Range(_) => 1,
        }
    }
}
