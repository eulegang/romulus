use std::collections::HashMap;

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

