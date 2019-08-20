//! A module which extracts romulus tokens out of string content

#[cfg(test)]
mod tests;
mod utils;

use utils::*;

///
/// Represents the individual grammer entity in romulus
///
#[derive(Debug, PartialEq)]
pub enum Token {
    /// Represents (, {, [, ], }, )
    Paren(char),

    /// Represents simple symbols like ^ and $
    Symbol(char),

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
    fn significant(&self) -> bool {
        match self {
            Token::Number(_) => true,
            Token::Paren(_) => true,
            Token::Regex(_, _) => true,
            Token::Comment(_) => false,
            Token::Identifier(_) => true,
            Token::String(_, _) => true,
            Token::Symbol(_) => true,
            Token::Newline => false,
            Token::Comma => true,
        }
    }
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
                let chars = chomp_until_escaped(&mut it, '/', false)?;
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
                let chars = chomp_until_escaped(&mut it, '"', true)?;
                it.next();
                let content = chars.iter().cloned().collect::<String>();

                tokens.push(Token::String(content, true));
            }

            '\'' => {
                it.next();
                let chars = chomp_until_escaped(&mut it, '\'', false)?;
                it.next();
                let content = chars.iter().cloned().collect::<String>();

                tokens.push(Token::String(content, false));
            }

            '_' | 'a'..='z' | 'A'..='Z' => {
                let chars = chomp_multi(&mut it, &['_'], &['a'..='z', 'A'..='Z', '0'..='9']);
                let content = chars.iter().cloned().collect::<String>();

                tokens.push(Token::Identifier(content));
            }

            '^' | '$' => {
                tokens.push(Token::Symbol(*ch));
                it.next();
            }

            a => {
                return Err(format!("unknown character: '{}'", a));
            }
        }
    }

    Ok(tokens)
}

