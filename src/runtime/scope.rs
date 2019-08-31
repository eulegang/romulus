use regex::{Captures, Regex};
use std::collections::HashMap;
use std::ops::{Add, AddAssign};

/// A scope containing variables
#[derive(Clone, Default)]
pub struct Scope {
    local: HashMap<String, String>,
}

impl Scope {
    /// Create a new empty scope
    pub fn new() -> Scope {
        Scope {
            local: HashMap::new(),
        }
    }

    /// Extracts a scope from a line given
    /// a regular expression with named capture groups
    pub fn from_regex(regex: &Regex, line: &str) -> Scope {
        let mut scope = Scope::new();

        if let Some(captures) = regex.captures(line) {
            for name in regex.capture_names() {
                if let Some(n) = name {
                    if n == "_" {
                        continue;
                    }

                    if let Some(m) = captures.name(n) {
                        scope.set(n.to_string(), m.as_str().to_string())
                    }
                }
            }
        }

        scope
    }

    pub(crate) fn from_captures(regex: &Regex, captures: &Captures) -> Scope {
        let mut scope = Scope::new();

        for name in regex.capture_names() {
            if let Some(n) = name {
                if n == "_" {
                    continue;
                }

                if let Some(m) = captures.name(n) {
                    scope.set(n.to_string(), m.as_str().to_string())
                }
            }
        }

        scope
    }
}

impl Scope {
    /// Sets a variable in the current scope
    pub fn set(&mut self, name: String, value: String) {
        self.local.insert(name, value);
    }

    /// Get a variable from the current scope
    pub fn get(&self, name: &str) -> Option<&String> {
        self.local.get(name)
    }
}

impl AddAssign for Scope {
    fn add_assign(&mut self, other: Scope) {
        for (key, value) in other.local {
            self.local.insert(key, value);
        }
    }
}

impl Add for Scope {
    type Output = Scope;
    fn add(self, other: Scope) -> Scope {
        let mut result = Scope::default();
        result += self;
        result += other;

        result
    }
}

impl Scope {
    /// Generate a scope from os environmental variables
    pub fn env() -> Scope {
        let mut scope = Scope::new();

        for (key, value) in std::env::vars() {
            scope.set(key, value);
        }

        scope
    }
}
