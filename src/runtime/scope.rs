use std::collections::HashMap;

/// A scope containing variables
#[derive(Clone)]
pub struct Scope {
    local: HashMap<String, String>,
}

impl Scope {

    /// Create a new empty scope
    pub fn new() -> Scope {
        Scope { local: HashMap::new() }
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

