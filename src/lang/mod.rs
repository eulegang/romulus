mod lex;
mod nodes;
mod func;
mod env;
mod ops;

use std::io::{BufRead,Write};
use env::Environment;
use ops::{Operation,RangeCap};

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
        let mut env = Environment::new(sout, self.node.num_ranges());

        while let Some(Ok(line)) = iter.next() {
            env.lineno += 1;
            env.line = line;

            self.node.perform(&mut env);
            env.reset_range();
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! run_interpreter {
        ($prog: expr, $input: expr) => {
            {
                let interpreter = Interpreter::new($prog).unwrap();
                let mut out = Vec::new();
                let mut sin = $input.as_bytes();

                interpreter.process(&mut sin, &mut out);
                String::from_utf8(out).unwrap()
            }
        }
    }

    #[test]
    fn basic() {
        assert_eq!(
            run_interpreter!(
                "/needle/ { print('found it') }",
                "hay\nhay\nhey\nneedle\nhay"
            ),
            "found it\n".to_string()
        );
    }

    #[test]
    fn range() {
        assert_eq!(
            run_interpreter!(
                "/third/,/fifth/ { print() }",
                "first\nsecond\nthird\nfourth\nfifth\nsexth\nseventh\nninth\ntenth\n"
            ),
            "third\nfourth\n"
        );
    }

    #[test]
    fn capture() {
        assert_eq!(
            run_interpreter!(
                "1 { print('name,type') }\n/pokemon \"(?P<name>.*)\"/,/}/ { /type *= *\"(?P<type>.*)\"/ { print(name, ',', type) } }",
                "pokemon \"Haunter\" {\ntype = \"Ghost\"\n}\npokemon \"Noctowl\" {\ntype = \"Flying\"\n}\n"
            ),
            "name,type\nHaunter,Ghost\nNoctowl,Flying\n"
        )
    }
}
