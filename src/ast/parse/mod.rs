use super::*;
use crate::lex::Token;

mod regex;

/// Parses a romulus token stream and creates a romulus AST,
/// or returns an error message
pub fn parse(tokens: Vec<Token>) -> Result<Seq, String> {
    let (node, offset) = Seq::parse_toplevel(&tokens, 0)?;

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
        let toplevel = false;

        while cur != tokens.len() {
            let (body_node, next) = Body::parse(&tokens, cur)?;
            cur = next;
            subnodes.push(body_node);
        }

        Ok((Seq { subnodes, toplevel }, cur))
    }
}

impl Seq {
    fn parse_toplevel(tokens: &[Token], pos: usize) -> Result<(Seq, usize), String> {
        let mut subnodes = Vec::new();
        let mut cur = pos;
        let toplevel = true;

        while cur != tokens.len() {
            let (body_node, next) = Body::parse(&tokens, cur)?;
            cur = next;
            subnodes.push(body_node);
        }

        Ok((Seq { subnodes, toplevel }, cur))
    }
}

impl Parsable for Body {
    fn parse(tokens: &[Token], pos: usize) -> Result<(Body, usize), String> {
        if let Some((sel, cur)) = Selector::try_parse(&tokens, pos) {
            if Some(&Token::Paren('{')) != tokens.get(cur) {
                let (statement, done) = Statement::parse(tokens, cur)?;

                return Ok((Body::Single(sel, statement), done))
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

            Ok((Body::Guard(sel, Seq { subnodes, toplevel: false }), current + 1))
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

            return Ok((Selector::Pattern(pattern_match), next));
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
            return Err(format!(
                "expected start to pattern match but received {:?}",
                token
            ));
        }

        if tokens.get(pos + 1) == Some(&Token::Paren(']')) {
            return Ok((PatternMatch { patterns: vec![] }, pos + 2));
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
        if let Some(Token::Regex(pattern, flags)) = tokens.get(pos) {
            let regex = regex::to_regex(pattern.to_string(), flags.to_string())?;
            return Ok((Pattern::Regex(regex), pos + 1));
        }

        if let Some(Token::String(content, interpolatable)) = tokens.get(pos) {
            return Ok((
                Pattern::String(content.to_string(), *interpolatable),
                pos + 1,
            ));
        }

        if let Some(Token::Identifier(name)) = tokens.get(pos) {
            return Ok((Pattern::Identifier(name.to_string()), pos + 1));
        };

        Err(format!(
            "Expected litteral or identifier but received: {:?}",
            tokens.get(pos)
        ))
    }
}

impl Parsable for Match {
    fn parse(tokens: &[Token], pos: usize) -> Result<(Match, usize), String> {
        let token = guard_eof!(tokens.get(pos));

        match token {
            Token::Number(num) => Ok((Match::Index(*num), pos + 1)),
            Token::Regex(pattern, flags) => {
                let regex = regex::to_regex(pattern.to_string(), flags.to_string())?;
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

impl Parsable for Statement {
    fn parse(tokens: &[Token], pos: usize) -> Result<(Statement, usize), String> {
        let token = guard_eof!(tokens.get(pos));

        let id = match token {
            Token::Identifier(id) => id,
            _ => return Err(format!("expected identifier but received {:?}", token)),
        };

        let parens = tokens.get(pos + 1) == Some(&Token::Paren('('));
        let param_pos = if parens { pos + 2 } else { pos + 1 };

        let (statement, end_pos) = match &id[..] {
            "print" => {
                let (expr, p) = Expression::parse(tokens, param_pos)?;
                (Statement::Print(expr), p)
            }
            "quit" => (Statement::Quit, param_pos),

            "subst" => {
                let regex = match tokens.get(param_pos) {
                    Some(Token::Regex(pat, flags)) => {
                        regex::to_regex(pat.to_string(), flags.to_string())?
                    }
                    _ => return Err(format!("expected a regex for subst but received {:?}", tokens.get(param_pos))),
                };

                if Some(&Token::Comma) != tokens.get(param_pos+1) {
                    return Err(format!("expected a comma but found {:?}", tokens.get(param_pos+1)));
                }

                let (expr, p) = Expression::parse(tokens, param_pos + 2)?;

                (Statement::Subst(regex, expr), p)
            }

            "gsubst" => {
                let regex = match tokens.get(param_pos) {
                    Some(Token::Regex(pat, flags)) => {
                        regex::to_regex(pat.to_string(), flags.to_string())?
                    }
                    _ => return Err(format!("expected a regex for subst but received {:?}", tokens.get(param_pos))),
                };

                if Some(&Token::Comma) != tokens.get(param_pos+1) {
                    return Err(format!("expected a comma but found {:?}", tokens.get(param_pos+1)));
                }

                let (expr, p) = Expression::parse(tokens, param_pos + 2)?;

                (Statement::Gsubst(regex, expr), p)
            }

            "read" => {
                let (expr, p) = Expression::parse(tokens, param_pos)?;
                (Statement::Read(expr), p)
            }

            "write" => {
                let (expr, p) = Expression::parse(tokens, param_pos)?;
                (Statement::Write(expr), p)
            }

            "exec" => {
                let (expr, p) = Expression::parse(tokens, param_pos)?;
                (Statement::Exec(expr), p)
            }

            "append" => {
                let (expr, p) = Expression::parse(tokens, param_pos)?;
                (Statement::Append(expr), p)
            }

            "set" => {
                let (expr, p) = Expression::parse(tokens, param_pos)?;
                (Statement::Set(expr), p)
            }

            _ => {
                return Err(format!(
                    "expected a valid statement but received invalid one {:?}",
                    id
                ))
            }
        };

        if parens && tokens.get(end_pos) != Some(&Token::Paren(')')) {
            return Err("unterminated statement".to_string());
        }

        Ok((statement, if parens { end_pos + 1 } else { end_pos }))
    }
}

impl Parsable for Expression {
    fn parse(tokens: &[Token], pos: usize) -> Result<(Expression, usize), String> {
        let token = guard_eof!(tokens.get(pos));

        if let Token::String(content, interpolatable) = token {
            return Ok((
                Expression::String(content.to_string(), *interpolatable),
                pos + 1,
            ));
        }

        if let Some(Token::Identifier(name)) = tokens.get(pos) {
            return Ok((Expression::Identifier(name.to_string()), pos + 1));
        };

        Err(format!(
            "Expected litteral or identifier but received: {:?}",
            tokens.get(pos)
        ))
    }
}

#[cfg(test)]
mod tests;

