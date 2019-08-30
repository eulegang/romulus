use crate::ast;

/// Test whether a program can perform a significant
/// operation during its run.
///
/// A significant operation may be
/// 1. reading a file
/// 2. writing to a file
/// 3. executing an external program
/// 4. print some variable
///
/// This trait is used to determine if autoprinting should
/// implicitly be turned on
pub(crate) trait SigStatement {
    fn significant(&self) -> bool;
}

impl SigStatement for ast::Seq {
    fn significant(&self) -> bool {
        for sub in &self.subnodes {
            if sub.significant() {
                return true;
            }
        }

        false
    }
}

impl SigStatement for ast::Body {
    fn significant(&self) -> bool {
        use ast::Body::*;

        match self {
            Bare(s) => s.significant(),
            Single(_, s) => s.significant(),
            Guard(_, s) => s.significant(),
        }
    }
}

impl SigStatement for ast::Statement {
    fn significant(&self) -> bool {
        use ast::Statement::*;

        match self {
            Print(_) => true,
            Read(_) => true,
            Write(_) => true,
            Exec(_) => true,
            _ => false,
        }
    }
}
