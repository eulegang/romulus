mod lex;
mod nodes;
mod func;

pub struct Interpreter {
    node: nodes::Node
}

impl Interpreter {
    fn new(buf: &str) -> Result<Interpreter, String> {
        let tokens = lex::lex(buf)?;
        let node = nodes::parse(tokens)?;

        return Ok(Interpreter{
            node: node
        })
    }
}

