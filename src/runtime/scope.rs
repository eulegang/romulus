use regex::{Captures, Regex};
use std::collections::HashMap;
use std::ops::{Add, AddAssign};

/// A scope containing variables
#[derive(Clone, Default, Debug)]
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
            for name in regex.capture_names().flatten() {
                if name == "_" {
                    continue;
                }

                if let Some(m) = captures.name(name) {
                    scope.set(name.to_string(), m.as_str().to_string())
                }
            }
        }

        scope
    }

    pub(crate) fn from_captures(regex: &Regex, captures: &Captures) -> Scope {
        let mut scope = Scope::new();

        for name in regex.capture_names().flatten() {
            if name == "_" {
                continue;
            }

            if let Some(m) = captures.name(name) {
                scope.set(name.to_string(), m.as_str().to_string())
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
        let local = std::env::vars().collect::<HashMap<String, String>>();

        Scope { local }
    }

    /// Picks specific from this scope to make a subscope
    pub fn pick(&self, keys: &[String]) -> Scope {
        let mut scope = Scope::new();

        for key in keys {
            if let Some(value) = self.local.get(key) {
                scope.set(key.to_owned(), value.to_owned());
            }
        }

        scope
    }
}
