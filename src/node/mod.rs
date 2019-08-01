use regex::Regex;

mod parse;

pub use parse::parse;

#[derive(Debug)]
pub enum Literal {
    Regex(Box<Regex>),
    String(String, bool),
}

#[derive(Debug)]
pub enum Match {
    Index(i64),
    Regex(Box<Regex>),
}

#[derive(Debug, PartialEq)]
pub struct Range(pub Match, pub Match);

#[derive(Debug, PartialEq)]
pub enum Selector {
    Match(Match),
    Range(Range),
}

#[derive(Debug, PartialEq)]
pub enum Expression {
    Literal(Literal),
    Identifier(String),
}

#[derive(Debug, PartialEq)]
pub struct Function {
    pub name: String,
    pub args: Vec<Expression>,
}

#[derive(Debug, PartialEq)]
pub enum Body {
    Bare(Function),
    Guard(Selector, Seq),
}

#[derive(Debug, PartialEq)]
pub struct Seq {
    pub subnodes: Vec<Body>,
}

impl PartialEq for Match {
    fn eq(&self, other: &Match) -> bool {
        if let (Match::Index(ai), Match::Index(bi)) = (self, other) {
            return ai == bi;
        }

        if let (Match::Regex(_), Match::Regex(_)) = (self, other) {
            return true;
        }

        false
    }
}

impl PartialEq for Literal {
    fn eq(&self, other: &Literal) -> bool {
        if let (Literal::Regex(_), Literal::Regex(_)) = (self, other) {
            return true;
        }

        if let (Literal::String(ss, si), Literal::String(os, oi)) = (self, other) {
            return ss == os && si == oi;
        }

        false
    }
}

