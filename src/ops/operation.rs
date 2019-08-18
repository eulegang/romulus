use super::*;
use crate::ast;
use crate::ops::lifecycle::Lifecycle;
use regex::Captures;
use std::io::{Write, copy};
use std::process::{Command, Stdio};


pub trait Operation {
    fn perform(&self, env: &mut Environment);
}

impl Operation for ast::Seq {
    fn perform(&self, env: &mut Environment) {
        if env.quit {
            return;
        }

        for sub in &self.subnodes {
            if !self.toplevel || env.event.is_lifecycle() == sub.is_lifecycle() {
                sub.perform(env)
            }
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
            ast::Body::Single(sel_node, node) => {
                if sel_node.select(env) {
                    env.push(sel_node.scope(env));
                    node.perform(env);
                    env.pop();
                }
            }
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
        use ast::Statement::*;
        use Event::*;

        if env.quit {
            return;
        }

        match self {
            Print(expr) => {
                let _ = write!(env.out, "{}{}", expr.to_value(env), nl!());
            }

            Quit => env.quit = true,

            Subst(regex, s) => {
                let line = match &env.event {
                    Line(line) => line.to_string(),
                    _ => return
                };

                env.event = Line(regex.replace(&line, |caps: &Captures|  {
                    env.push(Scope::from_captures(regex, caps));
                    let result = s.to_value(env);
                    env.pop();
                    result
                }).into_owned());
            }

            Gsubst(regex, s) => {
                let line = match &env.event {
                    Line(line) => line.to_string(),
                    _ => return
                };

                env.event = Line(regex.replace_all(&line, |caps: &Captures| { 
                    env.push(Scope::from_captures(regex, caps));
                    let result = s.to_value(env);
                    env.pop();

                    result
                }).into_owned());
            }

            Read(expr) => {
                let mut file = match std::fs::File::open(expr.to_value(env)) {
                    Ok(f) => f,
                    Err(msg) => {
                        eprint!("Error open file {}{}", msg, nl!());
                        return
                    }
                };

                match copy(&mut file, env.out) {
                    Ok(_) => (),
                    Err(msg) => {
                        eprint!("Error cating file {}{}", msg, nl!());
                    }
                }
            }

            Write(expr) => {
                if let Line(line) = &env.event {
                    let mut file = match std::fs::OpenOptions::new()
                        .write(true)
                        .append(true)
                        .create(true).open(expr.to_value(env)) {
                            Ok(f) => f,
                            Err(msg) => {
                                eprint!("Error oppening file {}{}", msg, nl!());
                                return;
                            }
                        };

                    match write!(file, "{}{}", line, nl!()) {
                        Ok(_) => (),
                        Err(msg) => {
                            eprint!("Error writing to file {}{}", msg, nl!());
                            return;
                        }
                    }
                }
            }

            Exec(expr) => {
                let r_child = if cfg!(not(target_os = "windows")) {
                    Command::new("sh")
                    .arg("-c")
                    .arg(expr.to_value(env))
                    .stdout(Stdio::piped())
                    .spawn()
                } else {
                    Command::new("cmd")
                    .arg("/C")
                    .arg(expr.to_value(env))
                    .stdout(Stdio::piped())
                    .spawn()
                };


                match r_child {
                    Err(msg) => {
                        eprint!("unable to execute: {}{}", msg, nl!());
                    }

                    Ok(child) => {
                        let _ = copy(&mut child.stdout.unwrap(), env.out);
                    }
                };
            }

            Append(expr) => {
                if let Line(line) = &env.event {
                    env.event = Line(format!("{}{}", line, expr.to_value(&env)));
                }
            }

            Set(expr) => {
                if let Line(_) = &env.event {
                    env.event = Line(expr.to_value(&env));
                }
            }
        }
    }
}
