//! A module which extracts romulus tokens out of string content

#[cfg(test)]
mod tests;
mod utils;

use utils::*;

///
/// Represents the individual grammer entity in romulus
///
#[derive(Debug, PartialEq)]
pub enum Token<'a> {
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
    Comment(&'a str),

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

impl Token<'_> {
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
    let mut it = buf.chars().enumerate().peekable();

    let lower = 'a'..='z';
    let upper = 'A'..='Z';
    let under_score = &['_'];
    let newline_chars = &['\n', '\r', ';'];
    let number_chars = '0'..='9';
    let regexflag_chars = &['i', 'U'];
    let x = [&lower, &upper, &number_chars];
    let ident_chars = (Multi(&x), under_score);

    while let Some((start, ch)) = it.peek() {
        let start = *start;
        match ch {
            '0'..='9' => {
                let end = chomp(&number_chars, &mut it);
                tokens.push(Token::Number(get_number(&buf[start..end])));
            }

            '{' | '[' | '(' | '}' | ']' | ')' => {
                tokens.push(Token::Paren(*ch));
                it.next();
            }

            ' ' | '\t' => {
                it.next();
            }

            '\n' | '\r' | ';' => {
                chomp(&newline_chars, &mut it);
                tokens.push(Token::Newline);
            }

            '#' => {
                it.next();
                let end = chomp_until(&newline_chars, &mut it);
                tokens.push(Token::Comment(&buf[start + 1..end]));
            }

            ',' => {
                it.next();
                tokens.push(Token::Comma);
            }

            '/' => {
                it.next();
                let chars = chomp_until_escaped(
                    &mut it,
                    '/',
                    &[
                        '{', '}', '[', ']', '.', '^', '$', '*', '+', '?', '|', '(', ')', 'd', 'D',
                        's', 'S', 'w', 'W', 'p', 'P', 'b', 'B', 'A', 'z', 'a', 'f', 't', 'n', 'r',
                        'v', 'x', 'u', 'U', '\\',
                    ],
                )?;
                let pattern = chars;
                if let Some((_, '/')) = it.next() {
                } else {
                    return Err("expected character: '/'".to_string());
                }

                let flags = chomp_str(&regexflag_chars, &mut it);

                tokens.push(Token::Regex(pattern, flags));
            }

            '"' => {
                it.next();
                let content = chomp_until_escaped(&mut it, '"', &['$'])?;
                it.next();

                tokens.push(Token::String(content, true));
            }

            '\'' => {
                it.next();
                let content = chomp_until_escaped(&mut it, '\'', &[])?;
                it.next();

                tokens.push(Token::String(content, false));
            }

            '_' | 'a'..='z' | 'A'..='Z' => {
                let content = chomp_str(&ident_chars, &mut it);

                tokens.push(Token::Identifier(content));
            }

            '^' | '$' | '!' | '&' | '|' => {
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
