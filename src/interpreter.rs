use crate::runtime::op::{Operation, SigStatement};
use crate::runtime::{Environment, Event, Scope};
use crate::{ast, lex, lint};

use regex::Regex;
use std::fs::File;
use std::io::{BufRead, Read, Write};
use std::path::Path;

/// The interpreter which processes lines with a romulus program
pub struct Interpreter {
    node: ast::Seq,
    sep: Regex,
    implicit_print: bool,
}

impl Interpreter {
    /// Creates a new interpreter with a string romulus program
    pub fn new<S: AsRef<str>>(
        buf: S,
        sep: Regex,
        implicit_print: bool,
    ) -> Result<Interpreter, String> {
        let tokens = lex::lex(buf.as_ref())?;
        let node = ast::parse(tokens)?;

        Ok(Interpreter {
            node,
            sep,
            implicit_print,
        })
    }

    /// Creates a new interpreter with a the contents of a file
    pub fn file<P: AsRef<Path>>(
        file: P,
        sep: Regex,
        implicit_print: bool,
    ) -> Result<Interpreter, String> {
        let mut file = match File::open(file.as_ref()) {
            Ok(f) => f,
            Err(err) => {
                return Err(format!(
                    "unable to open file romulus file '{}': {}",
                    file.as_ref().display(),
                    err
                ))
            }
        };

        let mut buf = String::new();
        if let Err(err) = file.read_to_string(&mut buf) {
            return Err(format!("unable to read romulus file: {}", err));
        }

        Interpreter::new(&buf, sep, implicit_print)
    }

    /// Process an input stream and writes the results for it's romulus program to
    /// the output stream
    pub fn process<R: BufRead, W: Write>(&self, sin: &mut R, sout: &mut W) {
        let mut iter = sin.lines();
        let mut env = Environment::new(sout, &self.node, self.sep.clone());

        if cfg!(feature = "envvar") {
            env.push(Scope::env());
        }

        let implicit_print = !self.node.significant();

        env.event = Event::Begin;
        self.node.perform(&mut env);

        while let Some(Ok(line)) = iter.next() {
            env.lineno += 1;
            env.event = Event::Line(line.to_string());

            self.node.perform(&mut env);

            if env.finished() {
                return;
            }

            if implicit_print && self.implicit_print {
                let line = match &env.event {
                    Event::Line(line) => line.clone(),
                    _ => unreachable!(),
                };

                env.print(&mut format!("{}{}", line, nl!()).as_bytes());
            }
        }

        env.event = Event::End;
        self.node.perform(&mut env);
    }

    /// Lint the current program
    pub fn lint(&self) -> Vec<lint::LintMessage> {
        lint::lint(&self.node)
    }
}
