use super::*;
use crate::ast;
use regex::Captures;

pub trait Operation {
    fn perform(&self, env: &mut Environment);
}

impl Operation for ast::Seq {
    fn perform(&self, env: &mut Environment) {
        if env.quit {
            return;
        }

        for sub in &self.subnodes {
            sub.perform(env)
        }
    }
}

impl Operation for ast::Body {
    fn perform(&self, env: &mut Environment) {
        if env.quit {
            return;
        }

        match self {
            ast::Body::Bare(func_node) => func_node.perform(env),
            ast::Body::Guard(sel_node, node) => {
                if sel_node.select(env) {
                    env.push(sel_node.scope(env));
                    node.perform(env);
                    env.pop();
                }
            }
        }
    }
}

impl Operation for ast::Statement {
    fn perform(&self, env: &mut Environment) {
        if env.quit {
            return;
        }

        match self {
            ast::Statement::Print(expr) => {
                let _ = writeln!(env.out, "{}", expr.to_value(env));
            }

            ast::Statement::Quit => env.quit = true,

            ast::Statement::Subst(regex, s) => {
                let line = match &env.event {
                    Event::Line(line) => line.to_string(),
                    _ => return
                };

                env.event = Event::Line(regex.replace(&line, |caps: &Captures|  {
                    env.push(Scope::from_captures(regex, caps));
                    let result = s.to_value(env);
                    env.pop();
                    result
                }).into_owned());
            }

            ast::Statement::Gsubst(regex, s) => {
                let line = match &env.event {
                    Event::Line(line) => line.to_string(),
                    _ => return
                };

                env.event = Event::Line(regex.replace_all(&line, |caps: &Captures| { 
                    env.push(Scope::from_captures(regex, caps));
                    let result = s.to_value(env);
                    env.pop();

                    result
                }).into_owned());
            }

            ast::Statement::Read(expr) => {
                let mut file = match std::fs::File::open(expr.to_value(env)) {
                    Ok(f) => f,
                    Err(msg) => {
                        eprintln!("Error open file {}", msg);
                        return
                    }
                };

                match std::io::copy(&mut file, env.out) {
                    Ok(_) => (),
                    Err(msg) => {
                        eprintln!("Error cating file {}", msg);
                    }
                }
            }
        }
    }
}
