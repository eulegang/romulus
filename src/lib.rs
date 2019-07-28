
#[macro_use]
extern crate lazy_static;

mod lex;
mod env;
mod func;
mod ops;
mod nodes;

use env::Environment;
use ops::{Operation, RangeCap};
use std::io::{BufRead, Write, Read};
use std::path::Path;
use std::fs::File;

pub struct Interpreter {
    node: nodes::Node,
}

impl Interpreter {
    pub fn new<S: AsRef<str>>(buf: S) -> Result<Interpreter, String> {
        let tokens = lex::lex(buf.as_ref())?;
        let node = nodes::parse(tokens)?;

        Ok(Interpreter { node })
    }

    pub fn file<P: AsRef<Path>>(file: P) -> Result<Interpreter, String> {
        let mut file = match File::open(file.as_ref()) {
            Ok(f) => f,
            Err(err) => return Err(format!("unable to open file romulus file: {}", err)),
        };

        let mut buf = String::new();
        if let Err(err) = file.read_to_string(&mut buf) {
            return Err(format!("unable to read romulus file: {}", err))
        }

        Interpreter::new(&buf)
    }

    pub fn process<R: BufRead, W: Write>(&self, sin: &mut R, sout: &mut W) {
        let mut iter = sin.lines();
        let mut env = Environment::new(sout, self.node.num_ranges());

        while let Some(Ok(line)) = iter.next() {
            env.lineno += 1;
            env.line = line;

            self.node.perform(&mut env);
            env.reset_range();
        }
    }
}

