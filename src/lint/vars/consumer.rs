use crate::ast::*;
use crate::lint::vars::provider::regex_provides;
use crate::runtime::op::interpolated_variables;

fn sub(base: Vec<String>, sub: Vec<String>) -> Vec<String> {
    let mut buf = Vec::with_capacity(base.len());

    for b in base {
        if !sub.contains(&b) {
            buf.push(b)
        }
    }
    buf
}

pub(super) trait ScopeConsumer {
    fn consumes(&self) -> Vec<String>;
}

impl ScopeConsumer for Statement {
    fn consumes(&self) -> Vec<String> {
        use Statement::*;

        match self {
            Print(expr) => expr.consumes(),
            Quit => vec![],
            Subst(regex, expr) => sub(expr.consumes(), regex_provides(regex)),
            Gsubst(regex, expr) => sub(expr.consumes(), regex_provides(regex)),
            Read(expr) => expr.consumes(),
            Write(expr) => expr.consumes(),
            Exec(expr) => expr.consumes(),
            Append(expr) => expr.consumes(),
            Set(expr) => expr.consumes(),

            #[cfg(feature = "bind")]
            Bind(id) => vec![id.to_string()],
        }
    }
}

impl ScopeConsumer for Expression {
    fn consumes(&self) -> Vec<String> {
        use Expression::*;
        match self {
            Identifier(name) => vec![name.to_string()],
            String(_, false) => vec![],

            String(content, true) => interpolated_variables(content),
        }
    }
}

pub(super) fn env_vars() -> Vec<String> {
    let mut vars = Vec::new();
    for (key, _) in std::env::vars() {
        vars.push(key)
    }
    vars
}
