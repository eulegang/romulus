
use std::io::{Write};
use std::collections::HashMap;
use super::func::{FunctionRegistry, Value};

pub struct Environment<'a> {
    pub lineno: i64,
    pub line: String,
    func_reg: FunctionRegistry,
    scope_stack: Vec<Scope>,
    out: &'a mut Write,
}

pub struct Scope {
    local: HashMap<String, String>,
}

impl <'a> Environment<'a> {
    pub fn new<W: Write>(w: &'a mut W) -> Environment<'a> {
        Environment{
            lineno: 1,
            line: String::new(),
            func_reg: FunctionRegistry::default(),
            scope_stack: Vec::new(),
            out: w,
        }
    }
}

impl <'a> Environment<'a> {
    pub fn lookup(&self, key: String) -> Option<String> {
        for scope in self.scope_stack.iter().rev() {
            if let Some(value) = scope.local.get(&key) { 
                return Some(value.to_string())
            }
        }

        None
    }

    pub fn push(&mut self, scope: Scope) {
        self.scope_stack.push(scope);
    }

    pub fn pop(&mut self) {
        self.scope_stack.pop();
    }

    pub(crate) fn call(&self, name: String, args: Vec<Value>) {
        match self.func_reg.call(name, args) {
            Ok(_) => (),
            Err(x) => panic!(x),
        }
    }
}

impl Scope {
    fn new() -> Scope {
        Scope{ local: HashMap::new() }
    }
}

impl <'a> Write for Environment<'a> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.out.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.out.flush()
    }
}
