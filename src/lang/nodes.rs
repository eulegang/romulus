
use super::lex::{Token};
use regex::Regex;

macro_rules! guard_eof {
    ($expr:expr) => {
        if let Some(token) = $expr {
            token
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

#[derive(Debug)]
pub(crate) enum LiteralNode {
    Regex(Box<Regex>),
    String(String, bool),
}

#[derive(Debug)]
pub(crate) enum MatchNode {
    Index(i64),
    Regex(Box<Regex>),
}

#[derive(Debug, PartialEq)]
pub(crate) struct RangeNode (pub MatchNode, pub MatchNode);

#[derive(Debug, PartialEq)]
pub(crate) enum SelectorNode {
    Match(MatchNode),
    Range(RangeNode),
}

#[derive(Debug, PartialEq)]
pub(crate) enum ExpressionNode {
    Literal(LiteralNode),
    Identifier(String),
}

#[derive(Debug, PartialEq)]
pub(crate) struct FunctionNode {
    pub(crate) name: String,
    pub(crate) args: Vec<ExpressionNode>
}

#[derive(Debug, PartialEq)]
pub (crate) enum BodyNode {
    Bare(FunctionNode),
    Guard(SelectorNode, Node),
}

#[derive(Debug, PartialEq)]
pub (crate) struct Node {
    pub(crate) subnodes: Vec<BodyNode>
}

impl PartialEq for MatchNode {
    fn eq(&self, other: &MatchNode) -> bool {
        if let (MatchNode::Index(ai), MatchNode::Index(bi)) = (self, other) {
            return ai == bi
        }

        if let (MatchNode::Regex(_), MatchNode::Regex(_)) = (self, other) {
            return true
        }

        return false
    }
}

impl PartialEq for LiteralNode {
    fn eq(&self, other: &LiteralNode) -> bool {
        if let (LiteralNode::Regex(_), LiteralNode::Regex(_)) = (self, other) {
            return true
        }

        if let (LiteralNode::String(ss, si), LiteralNode::String(os, oi)) = (self, other) {
            return ss == os && si == oi
        }

        return false
    }
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
            if Some(&Token::Paren('{')) != tokens.get(cur) {
                return Err(format!("expected {{ but received: {:?}", tokens.get(cur)));
            }

            let mut current = cur + 1;
            let mut subnodes = Vec::new();
            while Some(&Token::Paren('}')) != tokens.get(current) {
                let (node, next) = BodyNode::parse(&tokens, current)?;
                subnodes.push(node);
                current = next;
            }

            if Some(&Token::Paren('}')) != tokens.get(current) {
                return Err(format!("expected }} but received: {:?}", tokens.get(current)));
            }

            Ok((BodyNode::Guard(sel, Node{subnodes: subnodes}), current+1))
        } else {
            let (node, next) = FunctionNode::parse(&tokens, pos)?;
            Ok((BodyNode::Bare(node), next))
        }
    }
}

impl Parsable for SelectorNode {
    fn parse(tokens: &Vec<Token>, pos: usize) -> Result<(SelectorNode, usize), String> {
        let (s, next) = MatchNode::parse(tokens, pos)?;

        if Some(&Token::Comma) != tokens.get(next) {
            return Ok((SelectorNode::Match(s), next));
        }

        let (e, after_end) = MatchNode::parse(tokens, next+1)?;

        Ok((SelectorNode::Range(RangeNode(s, e)), after_end))
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

            _ => Err(format!("expected a regex or a number but received {:?}", token))
        }
    }
}

impl Parsable for RangeNode {
    fn parse(tokens: &Vec<Token>, pos: usize) -> Result<(RangeNode, usize), String> {
        let (start_match, after_start) = MatchNode::parse(&tokens, pos)?;

        if Some(&Token::Comma) != tokens.get(after_start) {
            return Err(format!("expected a comma but received: {:?}", tokens.get(after_start)));
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

            _ => Err(format!("expected a literal token but received {:?}", token)),
        }
    }
}

impl Parsable for FunctionNode {
    fn parse(tokens: &Vec<Token>, pos: usize) -> Result<(FunctionNode, usize), String> {
        let token  = guard_eof!(tokens.get(pos));

        let identifier = match token {
            Token::Identifier(name) => name,
            _ => return Err(format!("expected identifier for function name but received: {:?}", token)),
        };

        if Some(&Token::Paren('(')) != tokens.get(pos+1) {
            return Err(format!("expected ( but received {:?}", tokens.get(pos+1)));
        }

        if Some(&Token::Paren(')')) == tokens.get(pos+2) {
            return Ok((FunctionNode { name: identifier.to_string(), args: Vec::new() }, pos + 3))
        }

        let mut args = Vec::new();
        let mut cur = pos + 2;
        loop {
            let (expr, after_expr) = ExpressionNode::parse(&tokens, cur)?;
            cur = after_expr;
            args.push(expr);

            if Some(&Token::Paren(')')) == tokens.get(cur) {
                break;
            }

            if Some(&Token::Comma) != tokens.get(cur) {
                return Err(format!("expected comma but received {:?}", tokens.get(cur)));
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

        if let Some(Token::Identifier(name)) = tokens.get(pos) {
            return Ok((ExpressionNode::Identifier(name.to_string()), pos+1));
        };

        Err(format!("Expected litteral or identifier but received: {:?}", tokens.get(pos)))

        //must_rewrap!(FunctionNode, ExpressionNode::Func, tokens, pos)
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

#[cfg(test)]
mod parse_tests {
    use super::super::lex::lex;
    use super::*;

    #[test]
    fn basic_parse() {
        let tokens = match lex("/needle/ { print('found it') }") {
            Ok(tokens) => tokens,
            Err(msg) => panic!(msg),
        };

        assert_eq!(parse(tokens), Ok(Node{ subnodes: vec![
            BodyNode::Guard(
                SelectorNode::Match(MatchNode::Regex(Box::new(Regex::new("needle").unwrap()))),
                Node {
                    subnodes: vec![
                        BodyNode::Bare(FunctionNode{
                            name: String::from("print"),
                            args: vec![ExpressionNode::Literal(LiteralNode::String("found it".to_string(), false))],
                        })
                    ]
                }
            )
        ]}));
    }

    #[test]
    fn basic_func() {
        let tokens = match lex("print('found it')") {
            Ok(tokens) => tokens,
            Err(msg) => panic!(msg),
        };

        assert_eq!(parse(tokens), Ok(Node{ subnodes: vec![
            BodyNode::Bare(FunctionNode{
                name: String::from("print"),
                args: vec![ExpressionNode::Literal(LiteralNode::String("found it".to_string(), false))],
            })
        ]}));

    }

    #[test]
    fn parse_range() {
        let tokens = match lex("/a/,/b/ { print() }") {
            Ok(tokens) => tokens,
            Err(msg) => panic!(msg)
        };

        assert_eq!(parse(tokens), Ok(Node{ subnodes: vec![
            BodyNode::Guard(
                SelectorNode::Range(RangeNode(
                        MatchNode::Regex(Box::new(Regex::new("a").unwrap())), 
                        MatchNode::Regex(Box::new(Regex::new("b").unwrap())))),
                Node {
                    subnodes: vec![
                        BodyNode::Bare(FunctionNode{
                            name: String::from("print"),
                            args: vec![],
                        })
                    ]
                }
            )
        ]}));
    }

    #[test]
    fn parse_identifiers() {
        let tokens = match lex("/Type: (?P<type>.*)/ { print(type) }") {
            Ok(tokens) => tokens,
            Err(msg) => panic!(msg),
        };

        assert_eq!(parse(tokens), Ok(Node{ subnodes: vec![
            BodyNode::Guard(
                SelectorNode::Match(MatchNode::Regex(Box::new(Regex::new("Type: (?P<type>.*)").unwrap()))),
                Node {
                    subnodes: vec![
                        BodyNode::Bare(FunctionNode{
                            name: String::from("print"),
                            args: vec![ExpressionNode::Identifier("type".to_string())],
                        })
                    ]
                }
            )
        ]}));
    }
}

