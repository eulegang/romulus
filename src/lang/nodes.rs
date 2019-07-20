
use super::lex::{Token};
use regex::Regex;

macro_rules! guard_eof {
    ($expr:expr) => {
        if let Some(thing) = $expr {
            thing
        } else {
            return Err(String::from("Unexpected EOF"));
        }
    }
}

macro_rules! try_rewrap {
    ($target: ty, $rewrite: expr, $tokens: expr, $pos: expr) => {
        if let Some((node, next_pos)) = <$target>::try_parse($tokens, $pos) {
            return Ok(($rewrite(node), next_pos))
        }
    } 
}

macro_rules! must_rewrap {
    ($target: ty, $rewrite: expr, $tokens: ident, $pos: ident) => {
        {
            let (node, next_pos) = <$target>::parse($tokens, $pos)?;
            Ok(($rewrite(node), next_pos))
        }
    } 
}

pub(crate) enum LiteralNode {
    Regex(Box<Regex>),
    String(String, bool),
}

pub(crate) enum MatchNode {
    Index(i64),
    Regex(Box<Regex>),
}

pub(crate) struct RangeNode (MatchNode, MatchNode);

pub(crate) enum SelectorNode {
    Match(MatchNode),
    Range(RangeNode),
}

pub(crate) enum ExpressionNode {
    Func(FunctionNode),
    Literal(LiteralNode),
}

pub(crate) struct FunctionNode {
    name: String,
    args: Vec<ExpressionNode>
}

pub (crate) enum BodyNode {
    Bare(ExpressionNode),
    Guard(SelectorNode, Node),
}

pub (crate) struct Node {
    subnodes: Vec<BodyNode>
}

pub (crate) fn parse(tokens: Vec<Token>) -> Result<Node, String> {
    let (node, offset) = Node::parse(&tokens, 0)?;

    if offset != tokens.len() {
        return Err("Did not consume the whole program".to_string())
    }

    return Ok(node)
}

trait Parsable: Sized {
    fn parse(tokens: &Vec<Token>, pos: usize) -> Result<(Self, usize), String>;
    fn try_parse(tokens: &Vec<Token>, pos: usize) -> Option<(Self, usize)> {
        Self::parse(&tokens, pos).ok()
    }
}

impl Parsable for Node {
    fn parse(tokens: &Vec<Token>, pos: usize) -> Result<(Node, usize), String> {
        let mut nodes = Vec::new();
        let mut cur = pos;

        while cur != tokens.len() {
            let (body_node, next) = BodyNode::parse(&tokens, cur)?;
            cur = next;
            nodes.push(body_node);
        }

        return Ok((Node{ subnodes: nodes }, cur))
    }
}

impl Parsable for BodyNode {
    fn parse(tokens: &Vec<Token>, pos: usize) -> Result<(BodyNode, usize), String> {
        if let Some((sel, cur)) = SelectorNode::try_parse(&tokens, pos) {
            let (node, next) = Node::parse(&tokens, cur)?;
            Ok((BodyNode::Guard(sel, node), next))
        } else {
            let (node, next) = ExpressionNode::parse(&tokens, pos)?;
            Ok((BodyNode::Bare(node), next))
        }
    }
}

impl Parsable for SelectorNode {
    fn parse(tokens: &Vec<Token>, pos: usize) -> Result<(SelectorNode, usize), String> {
        try_rewrap!(MatchNode, SelectorNode::Match, tokens, pos);

        let (range_node, cur) = RangeNode::parse(&tokens, pos)?;
        Ok((SelectorNode::Range(range_node), cur))
    }
}

impl Parsable for MatchNode {
    fn parse(tokens: &Vec<Token>, pos: usize) -> Result<(MatchNode, usize), String> {
        let token = guard_eof!(tokens.get(pos));

        match token {
            Token::Number(num) => 
                Ok((MatchNode::Index(*num), pos + 1)),
            Token::Regex(pattern, flags) => {
                let regex = to_regex(pattern.to_string(), flags.to_string())?;
                Ok((MatchNode::Regex(regex), pos + 1))
            }

            _ => Err(format!("Expected a regex or a number but received something else"))
        }
    }
}

impl Parsable for RangeNode {
    fn parse(tokens: &Vec<Token>, pos: usize) -> Result<(RangeNode, usize), String> {
        let (start_match, after_start) = MatchNode::parse(&tokens, pos)?;

        if Some(&Token::Comma) != tokens.get(after_start) {
            return Err(String::from("expected a comma"));
        }

        let (end_match, after_end) = MatchNode::parse(&tokens, after_start+1)?;

        Ok((RangeNode(start_match, end_match), after_end))
    }
}

impl Parsable for LiteralNode {
    fn parse(tokens: &Vec<Token>, pos: usize) -> Result<(LiteralNode, usize), String> {
        let token = guard_eof!(tokens.get(pos));

        match token {
            Token::Regex(pattern, flags) => {
                let regex = to_regex(pattern.to_string(), flags.to_string())?;
                Ok((LiteralNode::Regex(regex), pos + 1))
            }

            Token::String(content, interpolatable) => {
                Ok((LiteralNode::String(content.to_string(), *interpolatable), pos+1))
            }

            _ => Err(format!("Expected a literal token")),
        }
    }
}

impl Parsable for FunctionNode {
    fn parse(tokens: &Vec<Token>, pos: usize) -> Result<(FunctionNode, usize), String> {
        let token  = guard_eof!(tokens.get(pos));

        let identifier = match token {
            Token::Identifier(name) => name,
            _ => return Err(String::from("Expected identifier")),
        };

        if Some(&Token::Paren('(')) != tokens.get(pos+1) {
            return Err("expected (".to_string());
        }

        if Some(&Token::Paren(')')) == tokens.get(pos+2) {
            return Ok((FunctionNode { name: identifier.to_string(), args: Vec::new() }, pos + 3))
        }

        let mut args = Vec::new();
        let mut cur = pos + 2;
        loop {
            let (expr, after_expr) = ExpressionNode::parse(&tokens, cur+1)?;
            cur = after_expr;
            args.push(expr);

            if Some(&Token::Paren(')')) == tokens.get(cur) {
                break;
            }

            if Some(&Token::Comma) != tokens.get(cur) {
                return Err("Expected comma and then next arg".to_string());
            }

            cur += 1;
        }

        Ok((FunctionNode {
            name: identifier.to_string(),
            args: args,
        }, cur + 1))
    }
}

impl Parsable for ExpressionNode {
    fn parse(tokens: &Vec<Token>, pos: usize) -> Result<(ExpressionNode, usize), String> {
        try_rewrap!(LiteralNode, ExpressionNode::Literal, tokens, pos);
        must_rewrap!(FunctionNode, ExpressionNode::Func, tokens, pos)
    }
}

fn to_regex(pat: String, flags: String) -> Result<Box<Regex>, String> {
    let prepend = if flags.is_empty() {
        String::new()
    } else {
        format!("(?{})", flags)
    };

    match regex::Regex::new(&format!("{}{}", prepend,  pat)) {
        Ok(regex) => Ok(Box::new(regex)),
        Err(_) => Err(format!("Can not create from /{}/{}", pat, flags)),
    }
}
