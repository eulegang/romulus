use crate::ops::Operation;
use crate::runtime::{Environment, Event};
use crate::{ast, lex};

use regex::Regex;
use std::fs::File;
use std::io::{BufRead, Read, Write};
use std::path::Path;

/// The interpreter which processes lines with a romulus program
pub struct Interpreter {
    node: ast::Seq,
    sep: Regex,
}

impl Interpreter {
    /// Creates a new interpreter with a string romulus program
    pub fn new<S: AsRef<str>>(buf: S, sep: Regex) -> Result<Interpreter, String> {
        let tokens = lex::lex(buf.as_ref())?;
        let node = ast::parse(tokens)?;

        Ok(Interpreter { node, sep })
    }

    /// Creates a new interpreter with a the contents of a file
    pub fn file<P: AsRef<Path>>(file: P, sep: Regex) -> Result<Interpreter, String> {
        let mut file = match File::open(file.as_ref()) {
            Ok(f) => f,
            Err(err) => return Err(format!("unable to open file romulus file: {}", err)),
        };

        let mut buf = String::new();
        if let Err(err) = file.read_to_string(&mut buf) {
            return Err(format!("unable to read romulus file: {}", err));
        }

        Interpreter::new(&buf, sep)
    }

    /// Process an input stream and writes the results for it's romulus program to
    /// the output stream
    pub fn process<R: BufRead, W: Write>(&self, sin: &mut R, sout: &mut W) {
        let mut iter = sin.lines();
        let mut env = Environment::new(sout, &self.node, self.sep.clone());

        env.event = Event::Begin;
        self.node.perform(&mut env);
        env.tracker.reset();

        while let Some(Ok(line)) = iter.next() {
            env.lineno += 1;
            env.event = Event::Line(line.to_string());

            self.node.perform(&mut env);
            env.tracker.reset();

            if env.quit {
                return;
            }
        }

        env.event = Event::End;
        self.node.perform(&mut env);
        env.tracker.reset();
    }
}
