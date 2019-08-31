use crate::ast;

//
// Specifies how many ranges need to be kept track of
//
pub(crate) trait RangeCap {
    fn num_ranges(&self) -> usize;
}

impl RangeCap for ast::Seq {
    fn num_ranges(&self) -> usize {
        let mut count = 0;
        for sub in &self.subnodes {
            count += sub.num_ranges();
        }

        count
    }
}

impl RangeCap for ast::Body {
    fn num_ranges(&self) -> usize {
        use ast::Body::*;
        match self {
            Bare(_) => 0,
            Single(sel, _) => sel.num_ranges(),
            Guard(sel, node) => sel.num_ranges() + node.num_ranges(),
        }
    }
}

impl RangeCap for ast::Selector {
    fn num_ranges(&self) -> usize {
        use ast::Selector::*;

        match self {
            Match(_) => 0,
            Range(_) => 1,
            Pattern(_) => 0,
            Negate(sub) => sub.num_ranges(),
            Conjunction(lh, rh) => lh.num_ranges() + rh.num_ranges(),
        }
    }
}
