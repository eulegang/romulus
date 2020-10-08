use crate::ast::*;

pub trait Globals {
    fn globals(&self) -> Vec<String>;
}

impl Globals for Statement {
    #[cfg(feature = "bind")]
    fn globals(&self) -> Vec<String> {
        match self {
            Statement::Bind(id) => vec![id.to_string()],
            _ => vec![],
        }
    }

    #[cfg(not(feature = "bind"))]
    fn globals(&self) -> Vec<String> {
        vec![]
    }
}

impl Globals for Seq {
    fn globals(&self) -> Vec<String> {
        let mut v = Vec::new();

        for s in &self.subnodes {
            v.extend(s.globals());
        }

        v
    }
}

impl Globals for Body {
    fn globals(&self) -> Vec<String> {
        let mut v = Vec::new();

        match self {
            Body::Bare(stmt) => v.extend(stmt.globals()),

            Body::Single(_, stmt) => v.extend(stmt.globals()),

            Body::Guard(_, seq) => v.extend(seq.globals()),
        }

        v
    }
}
