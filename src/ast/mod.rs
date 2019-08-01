//! A module organizing the romulus abstract syntax tree

use regex::Regex;

mod parse;

pub use parse::parse;

/// A literal AST node 
#[derive(Debug)]
pub enum Literal {
    /// A regular expression literal
    Regex(Box<Regex>),

    /// A string expression literal
    String(String, bool),
}

/// A match node which guard a body statement
///
/// one of the basic concepts in romulus is to trigger 
/// a body of code when a condition is meet. 
///
/// Here is the basic form
///
/// ```
/// <match condition> {
///   <actions>
/// }
/// ```
///
#[derive(Debug)]
pub enum Match {
    /// The case to run a statements when a line number is reached
    /// 
    /// ```
    /// 1 {
    ///   print("Begin of input")
    /// }
    /// ```
    Index(i64),

    /// The case to run statements when a line matches a regex
    ///
    /// ```
    /// /(?P<type>struct|enum) +(?P<name>[_a-z0-9]+)/ {
    ///     print("${name} is a ${type}")
    /// }
    /// ```
    ///
    /// This program not only will print when romulus sees "struct SomeType",
    /// but it will also extract variables form the line and allow the statements to use
    /// elements of the line freely.
    ///
    /// This also nests when statements in the body are matches as well.
    Regex(Box<Regex>),
}

/// A range has two matches seperated by a comma
/// When the first one is matched all of the lines until the end is matched will
/// execute the body statement.
///
/// Ranges are start inclusive but end exclusive
///
/// When a start match is a regex and has capture variables it's variables are stored and 
/// supplied for each next until the range ends
///
/// ```
/// /start: (?P<type>.*)/, /end/{
///   /elem: (?P<elem>.*)/ {
///     print("${type}: ${elem}")
///   }
/// }
/// ```
///
#[derive(Debug, PartialEq)]
pub struct Range(pub Match, pub Match);

/// A selector is a switch for a guard
#[derive(Debug, PartialEq)]
pub enum Selector {
    /// A match is given
    ///
    /// ```
    /// 1 {
    ///   print("round one, fight!")
    /// }
    /// ```
    Match(Match),

    /// A range is given
    /// ```
    /// /BEGIN/,/END/ {
    ///   print() # would print initial BEGIN line
    /// }
    /// ```
    Range(Range),
}

/// A expression
#[derive(Debug, PartialEq)]
pub enum Expression {
    /// A litteral value
    Literal(Literal),

    /// A variable to be resolved
    Identifier(String),
}

/// A function call
#[derive(Debug, PartialEq)]
pub struct Function {
    pub name: String,
    pub args: Vec<Expression>,
}

/// A guarded statement or a plain one
#[derive(Debug, PartialEq)]
pub enum Body {
    Bare(Function),
    Guard(Selector, Seq),
}

/// Contains multiple sub nodes
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

