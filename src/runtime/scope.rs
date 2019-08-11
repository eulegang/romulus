use std::collections::HashMap;
use regex::Regex;
use std::ops::AddAssign;

/// A scope containing variables
#[derive(Clone)]
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

    pub fn from_regex(regex: &Regex, line: &str) -> Scope {
        let mut scope = Scope::new();

        if let Some(capture) = regex.captures(line) {
            for name in regex.capture_names() {
                if let Some(n) = name {
                    if n == "_" {
                        continue;
                    }

                    if let Some(m) = capture.name(n) {
                        scope.set(n.to_string(), m.as_str().to_string())
                    }
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
