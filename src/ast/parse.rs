use crate::lex::Token;
use super::*;

/// Parses a romulus token stream and creates a romulus AST,
/// or returns an error message
pub fn parse(tokens: Vec<Token>) -> Result<Seq, String> {
    let (node, offset) = Seq::parse(&tokens, 0)?;

    if offset != tokens.len() {
        return Err("Did not consume the whole program".to_string());
    }

    Ok(node)
}

macro_rules! guard_eof {
    ($expr:expr) => {
        if let Some(token) = $expr {
            token
        } else {
            return Err(String::from("Unexpected EOF"));
        }
    };
}

macro_rules! try_rewrap {
    ($target: ty, $rewrite: expr, $tokens: expr, $pos: expr) => {
        if let Some((node, next_pos)) = <$target>::try_parse($tokens, $pos) {
            return Ok(($rewrite(node), next_pos));
        }
    };
}

trait Parsable: Sized {
    fn parse(tokens: &[Token], pos: usize) -> Result<(Self, usize), String>;
    fn try_parse(tokens: &[Token], pos: usize) -> Option<(Self, usize)> {
        Self::parse(&tokens, pos).ok()
    }
}

impl Parsable for Seq {
    fn parse(tokens: &[Token], pos: usize) -> Result<(Seq, usize), String> {
        let mut subnodes = Vec::new();
        let mut cur = pos;

        while cur != tokens.len() {
            let (body_node, next) = Body::parse(&tokens, cur)?;
            cur = next;
            subnodes.push(body_node);
        }

        Ok((Seq { subnodes }, cur))
    }
}

impl Parsable for Body {
    fn parse(tokens: &[Token], pos: usize) -> Result<(Body, usize), String> {
        if let Some((sel, cur)) = Selector::try_parse(&tokens, pos) {
            if Some(&Token::Paren('{')) != tokens.get(cur) {
                return Err(format!("expected {{ but received: {:?}", tokens.get(cur)));
            }

            let mut current = cur + 1;
            let mut subnodes = Vec::new();
            while Some(&Token::Paren('}')) != tokens.get(current) {
                let (node, next) = Body::parse(&tokens, current)?;
                subnodes.push(node);
                current = next;
            }

            if Some(&Token::Paren('}')) != tokens.get(current) {
                return Err(format!(
                        "expected }} but received: {:?}",
                        tokens.get(current)
                        ));
            }

            Ok((Body::Guard(sel, Seq { subnodes }), current + 1))
        } else {
            let (node, next) = Statement::parse(&tokens, pos)?;
            Ok((Body::Bare(node), next))
        }
    }
}

impl Parsable for Selector {
    fn parse(tokens: &[Token], pos: usize) -> Result<(Selector, usize), String> {
        if tokens.get(pos) == Some(&Token::Paren('[')) {
            let (pattern_match, next) = PatternMatch::parse(tokens, pos)?;

            return Ok((Selector::Pattern(pattern_match), next))
        }

        let (s, next) = Match::parse(tokens, pos)?;

        if Some(&Token::Comma) != tokens.get(next) {
            return Ok((Selector::Match(s), next));
        }

        let (e, after_end) = Match::parse(tokens, next + 1)?;

        Ok((Selector::Range(Range(s, e)), after_end))
    }
}

impl Parsable for PatternMatch {
    fn parse(tokens: &[Token], pos: usize) -> Result<(PatternMatch, usize), String> {
        let token = guard_eof!(tokens.get(pos));

        if token != &Token::Paren('[') {
            return Err(format!("expected start to pattern match but received {:?}", token))
        }

        if tokens.get(pos+1) == Some(&Token::Paren(']')) {
            return Ok((PatternMatch { patterns: vec![] }, pos + 2))
        }

        let mut patterns = Vec::new();
        let mut cur = pos + 1;

        loop {
            let (pattern, after_pat) = Pattern::parse(&tokens, cur)?;
            cur = after_pat;
            patterns.push(pattern);

            if Some(&Token::Paren(']')) == tokens.get(cur) {
                break;
            }

            if Some(&Token::Comma) != tokens.get(cur) {
                return Err(format!("expected comma but received {:?}", tokens.get(cur)));
            }

            cur += 1;
        }

        Ok((PatternMatch { patterns }, cur + 1))
    }
}

impl Parsable for Pattern {
    fn parse(tokens: &[Token], pos: usize) -> Result<(Pattern, usize), String> {
        try_rewrap!(Literal, Pattern::Literal, tokens, pos);

        if let Some(Token::Identifier(name)) = tokens.get(pos) {
            return Ok((Pattern::Identifier(name.to_string()), pos + 1));
        };

        Err(format!("Expected litteral or identifier but received: {:?}", tokens.get(pos)))
    }
}

impl Parsable for Match {
    fn parse(tokens: &[Token], pos: usize) -> Result<(Match, usize), String> {
        let token = guard_eof!(tokens.get(pos));

        match token {
            Token::Number(num) => Ok((Match::Index(*num), pos + 1)),
            Token::Regex(pattern, flags) => {
                let regex = to_regex(pattern.to_string(), flags.to_string())?;
                Ok((Match::Regex(regex), pos + 1))
            }
            Token::Symbol('^') => Ok((Match::Begin, pos + 1)),
            Token::Symbol('$') => Ok((Match::End, pos + 1)),

            _ => Err(format!(
                    "expected a regex or a number but received {:?}",
                    token
                    )),
        }
    }
}

impl Parsable for Range {
    fn parse(tokens: &[Token], pos: usize) -> Result<(Range, usize), String> {
        let (start_match, after_start) = Match::parse(&tokens, pos)?;

        if Some(&Token::Comma) != tokens.get(after_start) {
            return Err(format!(
                    "expected a comma but received: {:?}",
                    tokens.get(after_start)
                    ));
        }

        let (end_match, after_end) = Match::parse(&tokens, after_start + 1)?;

        Ok((Range(start_match, end_match), after_end))
    }
}

impl Parsable for Literal {
    fn parse(tokens: &[Token], pos: usize) -> Result<(Literal, usize), String> {
        let token = guard_eof!(tokens.get(pos));

        match token {
            Token::Regex(pattern, flags) => {
                let regex = to_regex(pattern.to_string(), flags.to_string())?;
                Ok((Literal::Regex(regex), pos + 1))
            }

            Token::String(content, interpolatable) => Ok((
                    Literal::String(content.to_string(), *interpolatable),
                    pos + 1,
                    )),

            _ => Err(format!("expected a literal token but received {:?}", token)),
        }
    }
}

impl Parsable for Function {
    fn parse(tokens: &[Token], pos: usize) -> Result<(Function, usize), String> {
        let token = guard_eof!(tokens.get(pos));

        let identifier = match token {
            Token::Identifier(name) => name,
            _ => {
                return Err(format!(
                        "expected identifier for function name but received: {:?}",
                        token
                        ))
            }
        };

        if Some(&Token::Paren('(')) != tokens.get(pos + 1) {
            return Err(format!("expected ( but received {:?}", tokens.get(pos + 1)));
        }

        if Some(&Token::Paren(')')) == tokens.get(pos + 2) {
            return Ok((
                    Function {
                        name: identifier.to_string(),
                        args: Vec::new(),
                    },
                    pos + 3,
                    ));
        }

        let mut args = Vec::new();
        let mut cur = pos + 2;
        loop {
            let (expr, after_expr) = Expression::parse(&tokens, cur)?;
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

        let name = identifier.to_string();

        Ok((Function { name, args }, cur + 1))
    }
}

impl Parsable for Statement {
    fn parse(tokens: &[Token], pos: usize) -> Result<(Statement, usize), String> {
        let token = guard_eof!(tokens.get(pos));

        let id = match token {
            Token::Identifier(id) => id,
            _ => return Err(format!("expected identifier but received {:?}", token)),
        };

        let parens = tokens.get(pos+1) == Some(&Token::Paren('('));
        let param_pos = if parens { pos + 2 } else { pos + 1 };

        let (statement, end_pos) = match &id[..] {
            "print" => { 
                let (expr, p) = Expression::parse(tokens, param_pos)?;
                (Statement::Print(expr), p)
            }
            _ => return Err(format!("expected a valid statement but received invalid one {:?}", id)),
        };

        if parens && tokens.get(end_pos) != Some(&Token::Paren(')')) {
            return Err("unterminated statement".to_string());
        }

        Ok((statement, if parens { end_pos + 1 } else { end_pos }))
    }
}

impl Parsable for Expression {
    fn parse(tokens: &[Token], pos: usize) -> Result<(Expression, usize), String> {
        try_rewrap!(Literal, Expression::Literal, tokens, pos);

        if let Some(Token::Identifier(name)) = tokens.get(pos) {
            return Ok((Expression::Identifier(name.to_string()), pos + 1));
        };

        Err(format!(
                "Expected litteral or identifier but received: {:?}",
                tokens.get(pos)
                ))

            //must_rewrap!(Function, Expression::Func, tokens, pos)
    }
}

fn to_regex(pat: String, flags: String) -> Result<Box<Regex>, String> {
    let prepend = if flags.is_empty() {
        String::new()
    } else {
        format!("(?{})", flags)
    };

    match regex::Regex::new(&format!("{}{}", prepend, pat)) {
        Ok(regex) => Ok(Box::new(regex)),
        Err(_) => Err(format!("Can not create from /{}/{}", pat, flags)),
    }
}

#[cfg(test)]
mod parse_tests {
    use crate::lex::lex;
    use super::*;

    #[test]
    fn basic_parse() {
        let tokens = match lex("/needle/ { print('found it') }") {
            Ok(tokens) => tokens,
            Err(msg) => panic!(msg),
        };

        assert_eq!(
            parse(tokens),
            Ok(Seq {
                subnodes: vec![Body::Guard(
                              Selector::Match(Match::Regex(Box::new(Regex::new("needle").unwrap()))),
                              Seq {
                                  subnodes: vec![Body::Bare(Statement::Print(Expression::Literal(
                                                        Literal::String("found it".to_string(), false))))]
                              }
                              )]
            })
            );
    }

    #[test]
    fn basic_statement() {
        let tokens = match lex("print('found it')") {
            Ok(tokens) => tokens,
            Err(msg) => panic!(msg),
        };

        assert_eq!(
            parse(tokens),
            Ok(Seq {
                subnodes: vec![Body::Bare(Statement::Print(Expression::Literal(
                                      Literal::String("found it".to_string(), false))))],
            })
            );
    }

    #[test]
    fn parse_range() {
        let tokens = match lex("/a/,/b/ { print _ }") {
            Ok(tokens) => tokens,
            Err(msg) => panic!(msg),
        };

        assert_eq!(
            parse(tokens),
            Ok(Seq {
                subnodes: vec![Body::Guard(
                              Selector::Range(Range(
                                      Match::Regex(Box::new(Regex::new("a").unwrap())),
                                      Match::Regex(Box::new(Regex::new("b").unwrap()))
                                      )),
                                      Seq {
                                          subnodes: vec![Body::Bare(Statement::Print(Expression::Identifier("_".to_string())))]
                                      }
                                      )]
            })
            );
    }

    #[test]
    fn parse_identifiers() {
        let tokens = match lex("/Type: (?P<type>.*)/ { print(type) }") {
            Ok(tokens) => tokens,
            Err(msg) => panic!(msg),
        };

        assert_eq!(
            parse(tokens),
            Ok(Seq {
                subnodes: vec![Body::Guard(
                              Selector::Match(Match::Regex(Box::new(
                                          Regex::new("Type: (?P<type>.*)").unwrap()
                                          ))),
                                          Seq {
                                              subnodes: vec![Body::Bare(Statement::Print(Expression::Identifier("type".to_string())))]
                                          }
                                          )]
            })
            );
    }

    #[test]
    fn parse_pattern_match() {
        let tokens = match lex("['<none>', _, id] { print(id) }") {
            Ok(tokens) => tokens,
            Err(msg) => panic!(msg),
        };

        assert_eq!(
            parse(tokens),
            Ok(Seq {
                subnodes: vec![Body::Guard(
                              Selector::Pattern(
                                  PatternMatch{ patterns: vec![
                                      Pattern::Literal(Literal::String("<none>".to_string(), false)),
                                      Pattern::Identifier("_".to_string()),
                                      Pattern::Identifier("id".to_string()),
                              ]}),
                              Seq {
                                  subnodes: vec![Body::Bare(Statement::Print(Expression::Identifier("id".to_string())))]
                              })]
            })
        );
    }
}
