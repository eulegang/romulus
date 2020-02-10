use super::op::Valuable;
use super::RangeCap;
use super::{RangeScopeTracker, Scope};
use crate::ast::Seq;
use regex::{Regex, Split};
use std::collections::HashMap;
use std::io::{copy, Read, Write};

/// An event to be processed
#[derive(PartialEq, Debug)]
pub enum Event {
    /// The beginning of processing
    Begin,

    /// A line to be processed
    Line(String),

    /// The end of processing
    End,
}

/// Embodies the current state of the program
///
pub struct Environment<'a> {
    /// The current line number being processed
    pub lineno: i64,

    /// Current event being handled
    pub event: Event,

    pub(crate) tracker: RangeScopeTracker,

    globals: HashMap<String, String>,

    seperator: Regex,
    scope_stack: Vec<Scope>,
    out: &'a mut dyn Write,
    quit: bool,
}

impl<'a> Environment<'a> {
    /// Creates a new environment
    pub fn new<W: Write>(w: &'a mut W, node: &Seq, seperator: Regex) -> Environment<'a> {
        Environment {
            lineno: 0,
            event: Event::Begin,
            scope_stack: Vec::new(),
            quit: false,
            seperator,
            out: w,
            globals: HashMap::new(),
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

        if let Some(value) = self.globals.get(key) {
            return Some(value.to_string());
        }

        None
    }

    pub(crate) fn push(&mut self, scope: Scope) {
        self.scope_stack.push(scope);
    }

    pub(crate) fn pop(&mut self) {
        self.scope_stack.pop();
    }

    pub(crate) fn print(&mut self, reader: &mut dyn Read) {
        let _ = copy(reader, self.out);
    }

    pub(crate) fn quit(&mut self) {
        self.quit = true
    }

    pub(crate) fn finished(&self) -> bool {
        self.quit
    }

    pub(crate) fn split_line<F, T>(&self, handle: F) -> Option<T>
    where
        F: Fn(&mut Split) -> T,
    {
        if let Event::Line(line) = &self.event {
            Some(handle(&mut self.seperator.split(line)))
        } else {
            None
        }
    }

    pub(crate) fn eval<V: Valuable>(&mut self, scope: Scope, val: &V) -> String {
        self.push(scope);
        let result = val.to_value(self);
        self.pop();
        result
    }

    pub(crate) fn replace_line<F>(&mut self, handle: F)
    where
        F: Fn(&mut Self, String) -> String,
    {
        let line = if let Event::Line(line) = &mut self.event {
            line.clone()
        } else {
            return;
        };

        self.event = Event::Line(handle(self, line));
    }

    pub(crate) fn print_event(&mut self) {
        if let Event::Line(line) = &self.event {
            let buf = format!("{}{}", line, nl!());
            let _ = copy(&mut buf.as_bytes(), self.out);
        }
    }

    pub(crate) fn bind_variable(&mut self, key: &str) {
        if let Some(value) = self.lookup(key) {
            self.globals.insert(key.to_string(), value);
        }
    }
}
