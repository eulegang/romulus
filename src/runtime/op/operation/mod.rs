use super::lifecycle::Lifecycle;
use super::*;

use crate::ast;
use regex::Captures;
use std::io::Write;
use std::process::{Command, Stdio};

mod stmt;
use stmt::*;

pub trait Operation {
    fn perform(&self, env: &mut Environment);
}

impl Operation for ast::Seq {
    fn perform(&self, env: &mut Environment) {
        if env.finished() {
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
        use ast::Body::*;
        if env.finished() {
            return;
        }

        match self {
            Bare(func_node) => func_node.perform(env),
            Single(sel_node, node) => {
                if sel_node.select(env) {
                    env.push(sel_node.scope(env));
                    node.perform(env);
                    env.pop();
                }
            }
            Guard(sel_node, node) => {
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

        if env.finished() {
            return;
        }

        match self {
            Print(expr) => print(expr, env),
            Quit => quit(env),
            Subst(regex, expr) => subst(regex, expr, env),
            Gsubst(regex, expr) => gsubst(regex, expr, env),
            Read(expr) => read(expr, env),
            Write(expr) => write(expr, env),
            Exec(expr) => exec(expr, env),
            Append(expr) => append(expr, env),
            Set(expr) => set(expr, env),
        }
    }
}
