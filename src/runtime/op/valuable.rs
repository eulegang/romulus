use super::*;
use crate::ast::*;
use regex::{Captures, Regex};

pub trait Valuable {
    fn to_value(&self, env: &Environment) -> String;
}

lazy_static! {
    static ref INTERPOLATOR: Regex = Regex::new(r"\$\{(?P<name>[a-zA-Z0-9_]+)\}").unwrap();
}

// Not really a good other place for this imo
pub fn interpolated_variables(s: &str) -> Vec<String> {
    let mut names = Vec::new();

    for caps in INTERPOLATOR.captures_iter(s) {
        names.push(caps["name"].to_string())
    }

    names
}

impl Valuable for Expression {
    fn to_value(&self, env: &Environment) -> String {
        match self {
            Expression::String(content, interpolatable) => {
                if *interpolatable {
                    interpolate(content, env)
                } else {
                    content.to_string()
                }
            }

            Expression::Identifier(name) => {
                if let Some(value) = env.lookup(name) {
                    value
                } else {
                    String::new()
                }
            }
        }
    }
}

pub fn interpolate(content: &str, env: &Environment) -> String {
    let intermediary = content.replace("\\$", "\0");
    let eval = |capture: &Captures| -> String {
        let key = String::from(&capture["name"]);
        env.lookup(&key).unwrap_or_default()
    };

    let evaled = INTERPOLATOR
        .replace_all(&intermediary, eval)
        .to_owned()
        .to_string();

    evaled.replace('\0', "$")
}
