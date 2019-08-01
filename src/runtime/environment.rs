use std::io::Write;
use super::{Scope, RangeScopeTracker, Value, FunctionRegistry};
use crate::node::Seq;
use crate::ops::RangeCap;

pub struct Environment<'a> {
    pub lineno: i64,
    pub line: String,
    pub(crate) reg: &'a FunctionRegistry,
    scope_stack: Vec<Scope>,
    pub out: &'a mut dyn Write,
    pub(crate) tracker: RangeScopeTracker,
}

impl<'a> Environment<'a> {
    pub fn new<W: Write>(w: &'a mut W, node: &Seq, reg: &'a FunctionRegistry) -> Environment<'a> {
        Environment {
            lineno: 0,
            line: String::new(),
            reg,
            scope_stack: Vec::new(),
            out: w,
            tracker: RangeScopeTracker::new(node.num_ranges()),
        }
    }
}

impl<'a> Environment<'a> {
    pub fn lookup(&self, key: &str) -> Option<String> {
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

    pub(crate) fn call(&mut self, name: String, args: Vec<Value>) {
        let func = match self.reg.get(&name) {
            Some(f) => f,
            None => panic!(format!("expected {} to be defined", name)),
        };

        (func.proc)(self, &args);
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
