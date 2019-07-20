mod lex;
mod nodes;
mod func;

use std::io::{BufRead,Write};

pub struct Interpreter {
    node: nodes::Node
}

impl Interpreter {
    pub fn new(buf: &str) -> Result<Interpreter, String> {
        let tokens = lex::lex(buf)?;
        let node = nodes::parse(tokens)?;

        return Ok(Interpreter{
            node: node
        })
    }

    pub fn process<R: BufRead, W: Write>(&self, sin: R, sout: W) {
        let mut iter = sin.lines();

        while let Some(line) = iter.next() {

        }
    }
}

