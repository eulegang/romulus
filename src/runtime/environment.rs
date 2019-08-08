use std::io::Write;
use super::{Scope, RangeScopeTracker};
use regex::Regex;
use crate::ast::Seq;
use crate::ops::RangeCap;

/// An event to be processed
#[derive(PartialEq)]
pub enum Event {
    /// The beginning of processing
    Begin,

    /// A line to be processed
    Line(String),

    /// The end of processing
    End
}

/// Embodies the current state of the program
///
pub struct Environment<'a> {
    /// The current line number being processed
    pub lineno: i64,

    /// Current event being handled
    pub event: Event,

    scope_stack: Vec<Scope>,

    pub(crate) seperator: Regex,

    /// Where prints should write
    pub out: &'a mut dyn Write,
    pub(crate) tracker: RangeScopeTracker,
}

impl<'a> Environment<'a> {
    /// Creates a new environment
    pub fn new<W: Write>(w: &'a mut W, node: &Seq, seperator: Regex) -> Environment<'a> {
        Environment {
            lineno: 0,
            event: Event::Begin,
            scope_stack: Vec::new(),
            seperator,
            out: w,
            tracker: RangeScopeTracker::new(node.num_ranges()),
        }
    }
}

impl<'a> Environment<'a> {
    /// Looks up a variable from the stack of scopes
    pub fn lookup(&self, key: &str) -> Option<String> {
        if key == "_" {
            match self.event {
                Event::Line(ref s) => return Some(s.clone()),
                _ => return Some(String::new()),
            };
        }

        for scope in self.scope_stack.iter().rev() {
            if let Some(value) = scope.get(key) {
                return Some(value.to_string());
            }
        }

        None
    }

    pub(crate) fn push(&mut self, scope: Scope) {
        self.scope_stack.push(scope);
    }

    pub(crate) fn pop(&mut self) {
        self.scope_stack.pop();
    }
}

impl<'a> Write for Environment<'a> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.out.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.out.flush()
    }
}
