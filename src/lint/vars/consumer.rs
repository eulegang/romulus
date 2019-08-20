use crate::runtime::op::interpolated_variables;
use crate::ast::*;

pub(super) trait ScopeConsumer {
    fn consumes(&self) -> Vec<String>;
}

impl ScopeConsumer for Statement {
    fn consumes(&self) -> Vec<String> {
        use Statement::*;

        match self {
            Print(expr) => expr.consumes(),
            Quit => vec![],
            Subst(_, expr) => expr.consumes(),
            Gsubst(_, expr) => expr.consumes(),
            Read(expr) => expr.consumes(),
            Write(expr) => expr.consumes(),
            Exec(expr) => expr.consumes(),
            Append(expr) => expr.consumes(),
            Set(expr) => expr.consumes(),
        }
    }
}

impl ScopeConsumer for Expression {
    fn consumes(&self) -> Vec<String> {
        use Expression::*;
        match self {
            Identifier(name) => vec![name.to_string()],
            String(_, false) => vec![],

            String(content, true) => {
                interpolated_variables(content)
            }
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

