use crate::ast;

//
// Specifies how many ranges need to be kept track of
//
pub trait RangeCap {
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
        match self {
            ast::Body::Bare(_) => 0,
            ast::Body::Guard(sel, node) => sel.num_ranges() + node.num_ranges(),
        }
    }
}

impl RangeCap for ast::Selector {
    fn num_ranges(&self) -> usize {
        match self {
            ast::Selector::Match(_) => 0,
            ast::Selector::Range(_) => 1,
        }
    }
}
