#![allow(clippy::new_without_default)]

#[macro_use]
extern crate lazy_static;

pub mod lex;
pub mod runtime;
mod ops;
pub mod ast;

use runtime::{Environment, FunctionRegistry};
use ops::Operation;
use std::io::{BufRead, Write, Read};
use std::path::Path;
use std::fs::File;

/// The interpreter which processes lines with a romulus program
pub struct Interpreter {
    node: ast::Seq,
    reg: FunctionRegistry,
}

impl Interpreter {
    /// Creates a new interpret with a string romulus program
    /// and a FunctionRegistry
    pub fn new<S: AsRef<str>>(buf: S, reg: FunctionRegistry) -> Result<Interpreter, String> {
        let tokens = lex::lex(buf.as_ref())?;
        let node = ast::parse(tokens)?;

        Ok(Interpreter { node, reg })
    }

    /// Creates a new interpret with a the contents of a file
    /// and a FunctionRegistry
    pub fn file<P: AsRef<Path>>(file: P, reg: FunctionRegistry) -> Result<Interpreter, String> {
        let mut file = match File::open(file.as_ref()) {
            Ok(f) => f,
            Err(err) => return Err(format!("unable to open file romulus file: {}", err)),
        };

        let mut buf = String::new();
        if let Err(err) = file.read_to_string(&mut buf) {
            return Err(format!("unable to read romulus file: {}", err))
        }

        Interpreter::new(&buf, reg)
    }

    /// Process an input stream and writes the results for it's romulus program to 
    /// the output stream
    pub fn process<R: BufRead, W: Write>(&self, sin: &mut R, sout: &mut W) {
        let mut iter = sin.lines();
        let mut env = Environment::new(sout, &self.node, &self.reg);

        while let Some(Ok(line)) = iter.next() {
            env.lineno += 1;
            env.line = line;

            self.node.perform(&mut env);
            env.tracker.reset();
        }
    }
}

