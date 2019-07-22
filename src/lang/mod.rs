mod lex;
mod nodes;
mod func;
mod env;

mod ops;

use std::io::{BufRead,Write};
use env::Environment;

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

    pub fn process<R: BufRead, W: Write>(&self, sin: &mut R, sout: &mut W) {
        let mut iter = sin.lines();
        let env = Environment::new(sout);

        while let Some(line) = iter.next() {
            
        }
    }
}



