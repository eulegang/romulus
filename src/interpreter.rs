use crate::runtime::op::{Operation, ScopePersister, SigStatement};
use crate::runtime::{Environment, Event, Scope};
use crate::{ast, lex, lint};

use regex::Regex;
use std::fs::File;
use std::io::{BufRead, Read, Write};

/// The interpreter which processes lines with a romulus program
pub struct Interpreter {
    node: ast::Seq,
    sep: Regex,
    implicit_print: bool,
}

/// Builds an interpreter
pub struct Builder {
    filename: Option<String>,
    expression: Option<String>,
    sep: Option<Regex>,
    print: Option<bool>,
}

impl Interpreter {
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

            self.node.persist_scope(&mut env);
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

    /// Create a new interpreter builder
    pub fn builder() -> Builder {
        Builder {
            filename: None,
            expression: None,
            sep: None,
            print: None,
        }
    }
}

impl Builder {
    /// Sets a filename to be read
    pub fn filename(&mut self, filename: String) -> &mut Self {
        self.filename = Some(filename);
        self
    }

    /// Sets an expression
    pub fn expression(&mut self, expression: String) -> &mut Self {
        self.expression = Some(expression);
        self
    }

    /// sets the seperator
    pub fn sep(&mut self, sep: Regex) -> &mut Self {
        self.sep = Some(sep);
        self
    }

    /// sets the implicit printing
    pub fn print(&mut self, print: bool) -> &mut Self {
        self.print = Some(print);
        self
    }

    /// Builds the interpreter
    pub fn build(&mut self) -> Result<Interpreter, String> {
        let node = match (&self.filename, &self.expression) {
            (None, None) => return Err(String::from("Neither an expression or a file was given")),
            (Some(_), Some(_)) => {
                return Err(String::from(
                    "Both expression and file should not be given at the same time",
                ))
            }

            (Some(ref file), None) => {
                let mut file = match File::open(file) {
                    Ok(f) => f,
                    Err(err) => {
                        return Err(format!(
                            "unable to open file romulus file '{}': {}",
                            file, err
                        ))
                    }
                };

                let mut buf = String::new();
                if let Err(err) = file.read_to_string(&mut buf) {
                    return Err(format!("unable to read romulus file: {}", err));
                }

                let tokens = lex::lex(buf.as_ref())?;
                ast::parse(tokens)?
            }
            (None, Some(expr)) => {
                let tokens = lex::lex(expr.as_ref())?;
                ast::parse(tokens)?
            }
        };

        let sep = self
            .sep
            .clone()
            .unwrap_or_else(|| Regex::new(" +").unwrap());

        let implicit_print = self.print.unwrap_or(true);

        Ok(Interpreter {
            node,
            sep,
            implicit_print,
        })
    }
}
