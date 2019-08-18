//! A module organizing the romulus abstract syntax tree

use regex::Regex;

mod parse;

pub use parse::parse;

/// A pattern match
///
/// ```text
/// ["some ${var}", _, /abc/, 'xyz']
/// ```
#[derive(Debug, PartialEq)]
pub struct PatternMatch {
    /// The sub patterns to be matched against
    pub patterns: Vec<Pattern>,
}

/// A sub pattern of a pattern match
#[derive(Debug)]
pub enum Pattern {
    /// Regex pattern
    Regex(Box<Regex>),

    /// String Literal pattern
    String(String, bool),

    /// Identifier to bind to
    Identifier(String),
}

/// A match node which guard a body statement
///
/// one of the basic concepts in romulus is to trigger
/// a body of code when a condition is meet.
///
/// Here is the basic form
///
/// ```text
/// <match condition> {
///   <actions>
/// }
/// ```
///
#[derive(Debug)]
pub enum Match {
    /// The case where the first line should be matched
    ///
    /// ```text
    /// ^ {
    ///   print("start!")
    /// }
    /// ```
    Begin,

    /// The case where the last line should be matched
    ///
    /// ```text
    /// $ {
    ///   print("end!")
    /// }
    /// ```
    End,

    /// The case to run a statements when a line number is reached
    ///
    /// ```text
    /// 1 {
    ///   print("Begin of input")
    /// }
    /// ```
    Index(i64),

    /// The case to run statements when a line matches a regex
    ///
    /// ```text
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
/// ```text
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
    /// ```text
    /// 1 {
    ///   print("round one, fight!")
    /// }
    /// ```
    Match(Match),

    /// A range is given
    /// ```text
    /// /BEGIN/,/END/ {
    ///   print() # would print initial BEGIN line
    /// }
    /// ```
    Range(Range),

    /// A pattern is given
    /// ```text
    /// [/none/, _, id] {
    ///   print()
    /// }
    /// ```
    Pattern(PatternMatch),
}

/// A expression
#[derive(Debug, PartialEq)]
pub enum Expression {
    /// A string expression literal
    String(String, bool),
    /// A variable to be resolved
    Identifier(String),
}

/// A statement
#[derive(Debug)]
pub enum Statement {
    /// Print the given expression
    Print(Expression),
    Quit,
    Subst(Box<Regex>, Expression),
    Gsubst(Box<Regex>, Expression),
    Read(Expression),
    Write(Expression),
    Exec(Expression),

    /// Appends the value of the expression to the line
    Append(Expression),

    /// Sets the current line to an expression
    Set(Expression),
}

/// A guarded statement or a plain one
#[derive(Debug, PartialEq)]
pub enum Body {
    Bare(Statement),
    Single(Selector, Statement),
    Guard(Selector, Seq),
}

/// Contains multiple sub nodes
#[derive(Debug, PartialEq)]
pub struct Seq {
    pub subnodes: Vec<Body>,
    pub(crate) toplevel: bool,
}

impl PartialEq for Match {
    fn eq(&self, other: &Match) -> bool {
        match (self, other) {
            (Match::Index(a), Match::Index(b)) => a == b,
            (Match::Regex(a), Match::Regex(b)) => a.to_string() == b.to_string(),
            (Match::Begin, Match::Begin) => true,
            (Match::End, Match::End) => true,
            _ => false,
        }
    }
}

impl PartialEq for Pattern {
    fn eq(&self, other: &Pattern) -> bool {
        match (self, other) {
            (Pattern::Regex(a), Pattern::Regex(b)) => a.to_string() == b.to_string(),
            (Pattern::String(ss, si), Pattern::String(os, oi)) => ss == os && si == oi,
            (Pattern::Identifier(a), Pattern::Identifier(b)) => a == b,

            _ => false,
        }
    }
}

impl PartialEq for Statement {
    fn eq(&self, other: &Statement) -> bool {
        match (self, other) {
            (Statement::Quit, Statement::Quit) => true,
            (Statement::Print(se), Statement::Print(oe)) => se == oe,
            (Statement::Subst(sr, se), Statement::Subst(or, oe)) => 
                sr.to_string() == or.to_string() && se == oe,
            (Statement::Gsubst(sr, se), Statement::Gsubst(or, oe)) => 
                sr.to_string() == or.to_string() && se == oe,
            (Statement::Read(se), Statement::Read(oe)) => se == oe,
            (Statement::Write(se), Statement::Write(oe)) => se == oe,
            (Statement::Exec(se), Statement::Exec(oe)) => se == oe,
            (Statement::Append(se), Statement::Append(oe)) => se == oe,
            (Statement::Set(se), Statement::Set(oe)) => se == oe,
            _ => false
        }
    }
}

