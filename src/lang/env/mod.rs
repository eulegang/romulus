use super::func::{FunctionRegistry, Value};
use std::collections::HashMap;
use std::io::Write;

pub struct Environment<'a> {
    pub lineno: i64,
    pub line: String,
    pub func_reg: FunctionRegistry,
    scope_stack: Vec<Scope>,
    pub out: &'a mut Write,

    range_states: Vec<Option<Scope>>,
    range_pos: usize,
}

impl<'a> Environment<'a> {
    pub fn new<W: Write>(w: &'a mut W, num_ranges: usize) -> Environment<'a> {
        Environment {
            lineno: 0,
            line: String::new(),
            func_reg: FunctionRegistry::default(),
            scope_stack: Vec::new(),
            out: w,

            range_states: vec![None; num_ranges],
            range_pos: 0,
        }
    }
}

impl<'a> Environment<'a> {
    pub fn range(&self) -> bool {
        self.range_states[self.range_pos].is_some()
    }

    pub fn next_range(&mut self) {
        self.range_pos = (self.range_pos + 1) % self.range_states.len();
    }

    pub fn range_scope(&self) -> &Option<Scope> {
        &self.range_states[self.range_pos]
    }

    pub fn set_range_state(&mut self, scope: Scope) {
        self.range_states[self.range_pos] = Some(scope)
    }

    pub fn clear_range_state(&mut self) {
        self.range_states[self.range_pos] = None
    }

    pub fn reset_range(&mut self) {
        self.range_pos = 0;
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

    pub fn push(&mut self, scope: Scope) {
        self.scope_stack.push(scope);
    }

    pub fn pop(&mut self) {
        self.scope_stack.pop();
    }

    pub(crate) fn call(&mut self, name: String, args: Vec<Value>) {
        let func = match self.func_reg.get(&name) {
            Some(f) => f,
            None => panic!(format!("expected {} to be defined", name)),
        };

        (func.proc)(self, &args);
    }
}

#[derive(Clone)]
pub struct Scope {
    local: HashMap<String, String>,
}

impl Scope {
    pub fn new() -> Scope {
        Scope {
            local: HashMap::new(),
        }
    }

    pub fn set(&mut self, name: String, value: String) {
        self.local.insert(name, value);
    }

    pub fn get(&self, name: &str) -> Option<&String> {
        self.local.get(name)
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
