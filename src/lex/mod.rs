//! A module which extracts romulus tokens out of string content

use std::iter::Peekable;
use std::ops::RangeInclusive;

///
/// Represents the individual grammer entity in romulus
///
#[derive(Debug, PartialEq)]
pub enum Token {

    /// Represents (, {, [, ], }, )
    Paren(char),

    /// Represents positive decimal numbers <br>
    /// such as `42`
    Number(i64),

    /// Represents a Regular Expression with flags in the form of /regex/flags <br>
    /// such as `/[a-z]+/i`
    /// 
    /// flags supported
    /// 1. `i` - case insensitive
    /// 2. `U` - swap greediness semantics
    Regex(String, String),

    /// Represents a comment
    ///
    /// Currently only line comments are supported <br>
    /// any characters after `#` are apart of a comment
    Comment(String),

    /// Represents a variable identifier
    ///
    /// any bare caracters that match `/[_a-z][_a-z0-9]*/i` <br>
    /// i.e. _the_answer_42
    Identifier(String),

    /// Represents a string
    ///
    /// single quotes may not interpolate variables, where as double qoutes
    /// may interpolate variables with a `${identifier}`
    ///
    /// such as `'some string'`, `"Ip Address: ${ip}"`
    String(String, bool),

    /// A newline character, carriage returen, or semicolon
    Newline,

    /// A comma
    Comma,
}

impl Token {
    #[allow(dead_code)]
    fn typename(&self) -> String {
        match self {
            Token::Number(_) => "number".to_string(),
            Token::Paren(ch) => format!("{}", ch),
            Token::Regex(_, _) => "regex".to_string(),
            Token::Comment(_) => "comment".to_string(),
            Token::Identifier(_) => "identifier".to_string(),
            Token::String(_, _) => "string".to_string(),
            Token::Newline => "newline".to_string(),
            Token::Comma => "comma".to_string(),
        }
    }

    fn significant(&self) -> bool {
        match self {
            Token::Number(_) => true,
            Token::Paren(_) => true,
            Token::Regex(_, _) => true,
            Token::Comment(_) => false,
            Token::Identifier(_) => true,
            Token::String(_, _) => true,
            Token::Newline => false,
            Token::Comma => true,
        }
    }
}

#[inline]
fn chomp_range<T: Iterator<Item = char>>(
    iter: &mut Peekable<T>,
    accept: RangeInclusive<char>,
) -> Vec<char> {
    let mut accepted = Vec::new();

    while let Some(ch) = &mut iter.peek() {
        if accept.contains(&ch.clone()) {
            accepted.push(ch.clone());
            iter.next();
        } else {
            break;
        }
    }

    accepted
}

#[inline]
fn chomp_multi<T: Iterator<Item = char>>(
    iter: &mut Peekable<T>,
    chars: &[char],
    accepts: &[RangeInclusive<char>],
) -> Vec<char> {
    let mut accepted = Vec::new();

    'base: while let Some(ch) = &mut iter.peek() {
        let owned = **ch;

        if chars.contains(&owned) {
            accepted.push(owned);
            iter.next();
            continue;
        }

        for accept in accepts {
            if accept.contains(&owned) {
                accepted.push(owned);
                iter.next();
                continue 'base;
            }
        }

        break;
    }

    accepted
}

#[inline]
fn chomp_set<T: Iterator<Item = char>>(iter: &mut Peekable<T>, accept: &[char]) -> Vec<char> {
    let mut accepted = Vec::new();

    while let Some(ch) = &mut iter.peek() {
        let owned: char = **ch;

        if accept.contains(&owned) {
            accepted.push(owned);
            iter.next();
        } else {
            break;
        }
    }

    accepted
}

#[inline]
fn chomp_until<T: Iterator<Item = char>>(iter: &mut Peekable<T>, terminator: char) -> Vec<char> {
    let mut accepted: Vec<char> = Vec::new();

    while let Some(ch) = &mut iter.peek() {
        let owned: char = **ch;

        if owned != terminator {
            accepted.push(owned);
            iter.next();
        } else {
            break;
        }
    }

    accepted
}

#[inline]
fn chomp_until_set<T: Iterator<Item = char>>(iter: &mut Peekable<T>, accept: &[char]) -> Vec<char> {
    let mut accepted: Vec<char> = Vec::new();

    while let Some(ch) = &mut iter.peek() {
        let owned: char = **ch;

        if !accept.contains(&owned) {
            accepted.push(owned);
            iter.next();
        } else {
            break;
        }
    }

    accepted
}

#[inline]
fn get_number(vec: Vec<char>) -> i64 {
    let mut buffer = 0;
    for ch in vec {
        let digit = ch.to_string().parse::<i64>().unwrap();

        buffer = buffer * 10 + digit;
    }

    buffer
}

/// Lexes a given string and returns only significant tokens in
/// a romulus program
///
/// for example newlines and comments are not significant for parsing
/// a romulus program
pub fn lex(buf: &str) -> Result<Vec<Token>, String> {
    let tokens = full_lex(buf)?;

    Ok(tokens
        .into_iter()
        .filter(|t| t.significant())
        .collect::<Vec<Token>>())
}


/// Lexes a given string and returns all tokens found
pub fn full_lex(buf: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let mut it = buf.chars().peekable();

    while let Some(ch) = it.peek() {
        match ch {
            '0'..='9' => {
                let nums = chomp_range(&mut it, '0'..='9');
                tokens.push(Token::Number(get_number(nums)));
            }

            '{' | '[' | '(' | '}' | ']' | ')' => {
                tokens.push(Token::Paren(*ch));
                it.next();
            }

            ' ' | '\t' => {
                it.next();
            }

            '\n' | '\r' | ';' => {
                chomp_set(&mut it, &['\n', '\r', ';']);
                tokens.push(Token::Newline);
            }

            '#' => {
                it.next();
                let comment_chars = chomp_until_set(&mut it, &['\n', '\n', ';']);
                tokens.push(Token::Comment(comment_chars.iter().collect::<String>()));
            }

            ',' => {
                it.next();
                tokens.push(Token::Comma);
            }

            '/' => {
                it.next();
                let chars = chomp_until(&mut it, '/');
                let pattern = chars.iter().cloned().collect::<String>();
                if Some('/') != it.next() {
                    return Err("expected character: '/'".to_string());
                }

                let flag_chars = chomp_set(&mut it, &['i', 'U']);
                let flags = flag_chars.iter().cloned().collect::<String>();

                tokens.push(Token::Regex(pattern, flags));
            }

            '"' => {
                it.next();
                let chars = chomp_until(&mut it, '"');
                it.next();
                let content = chars.iter().cloned().collect::<String>();

                tokens.push(Token::String(content, true));
            }

            '\'' => {
                it.next();
                let chars = chomp_until(&mut it, '\'');
                it.next();
                let content = chars.iter().cloned().collect::<String>();

                tokens.push(Token::String(content, false));
            }

            '_' | 'a'..='z' | 'A'..='Z' => {
                let chars = chomp_multi(&mut it, &['_'], &['a'..='z', 'A'..='Z', '0'..='9']);
                let content = chars.iter().cloned().collect::<String>();

                tokens.push(Token::Identifier(content));
            }

            a => {
                return Err(format!("unknown character: '{}'", a));
            }
        }
    }

    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lex_numbers() {
        assert_eq!(full_lex("1234"), Ok(vec![Token::Number(1234)]));
    }

    #[test]
    fn test_lex_ignores_white_space() {
        assert_eq!(
            full_lex("\n \t1234 1\n\r"),
            Ok(vec![
                Token::Newline,
                Token::Number(1234),
                Token::Number(1),
                Token::Newline
            ])
        );
    }

    #[test]
    fn test_lex_context() {
        assert_eq!(
            full_lex("1 { }"),
            Ok(vec![Token::Number(1), Token::Paren('{'), Token::Paren('}')])
        );
    }

    #[test]
    fn test_lex_regex() {
        assert_eq!(
            full_lex("/.*/iU"),
            Ok(vec![Token::Regex(".*".to_string(), "iU".to_string())])
        );
    }

    #[test]
    fn test_lex_comment() {
        let tokens = vec![
            Token::Number(42),
            Token::Comment("some comment".to_string()),
            Token::Newline,
            Token::Regex("test *".to_string(), "i".to_string()),
        ];

        assert_eq!(full_lex("42 #some comment\n  /test */i"), Ok(tokens));
    }

    #[test]
    fn test_func() {
        let tokens = vec![
            Token::Identifier("subst".to_string()),
            Token::Paren('('),
            Token::Regex("(?P<name>[a-z]+):".to_string(), "".to_string()),
            Token::Comma,
            Token::String("Name: ${name}".to_string(), true),
            Token::Paren(')'),
        ];

        assert_eq!(
            full_lex("subst(/(?P<name>[a-z]+):/,\"Name: ${name}\")"),
            Ok(tokens)
        );
    }

    #[test]
    fn test_removed() {
        let tokens = vec![
            Token::Number(42),
            Token::Regex("test *".to_string(), "i".to_string()),
        ];

        assert_eq!(lex("42 #some comment\n  /test */i"), Ok(tokens));
    }
}
