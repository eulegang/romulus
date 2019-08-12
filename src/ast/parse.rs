use super::*;
use crate::lex::Token;

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
            let regex = to_regex(pattern.to_string(), flags.to_string())?;
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
                        to_regex(pat.to_string(), flags.to_string())?
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
                        to_regex(pat.to_string(), flags.to_string())?
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
    use super::*;
    use crate::lex::lex;

    macro_rules! seq {
        ($($ast: expr),*) => {
            {
                let mut subnodes = Vec::new();

                $(
                    subnodes.push($ast);
                )*

                Seq { subnodes }
            }
        }
    }

    macro_rules! quote {
        (s$ast: expr) => {
            Expression::String($ast.to_string(), false)
        };
        ($ast: expr) => {
            Expression::String($ast.to_string(), true)
        };
    }

    macro_rules! rmatch {
        ($ast: expr) => {
            Match::Regex(Box::new(Regex::new($ast).unwrap()))
        };
    }

    macro_rules! id {
        ($ast: expr) => {
            Expression::Identifier($ast.to_string())
        };
    }

    macro_rules! selector {
        (m$ast: expr) => { Selector::Match($ast) };
        (-$start:expr, $end:expr) => { Selector::Range(Range($start, $end)) };
        ($($ast: expr),*) => {
            {
                let mut patterns = Vec::new();

                $(
                    patterns.push($ast);
                )*

                Selector::Pattern(PatternMatch { patterns })
            }
        }
    }

    #[test]
    fn basic_parse() {
        let tokens = match lex("/needle/ { print('found it') }") {
            Ok(tokens) => tokens,
            Err(msg) => panic!(msg),
        };

        assert_eq!(
            parse(tokens),
            Ok(seq![Body::Guard(
                selector![m rmatch!("needle")],
                seq![Body::Bare(Statement::Print(quote![s"found it"]))]
            )])
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
            Ok(seq![Body::Bare(Statement::Print(quote![s"found it"]))])
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
            Ok(seq![Body::Guard(
                selector![-rmatch!("a"), rmatch!("b")],
                seq![Body::Bare(Statement::Print(id!("_")))]
            )])
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
            Ok(seq![Body::Guard(
                selector![m rmatch!("Type: (?P<type>.*)")],
                seq![Body::Bare(Statement::Print(id!("type")))]
            )])
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
            Ok(seq![Body::Guard(
                selector![
                    Pattern::String("<none>".to_string(), false),
                    Pattern::Identifier("_".to_string()),
                    Pattern::Identifier("id".to_string())
                ],
                seq![Body::Bare(Statement::Print(id!("id")))]
            )])
        );
    }

    #[test]
    fn parse_statement_patterns() {
        let tokens = match lex("['DONE'] { quit }\n/thing/{print _}") {
            Ok(tokens) => tokens,
            Err(msg) => panic!(msg),
        };

        assert_eq!(
            parse(tokens),
            Ok(seq![
                Body::Guard(
                    selector!(Pattern::String("DONE".to_string(), false)),
                    seq![Body::Bare(Statement::Quit)]
                ),
                Body::Guard(
                    selector![m rmatch!("thing")],
                    seq![Body::Bare(Statement::Print(id!("_")))]
                )
            ])
        );
    }

    #[test]
    fn parse_statement_subst() {
        let tokens = match lex("/thing/ { subst /that/, 'other' }") {
            Ok(tokens) => tokens,
            Err(msg) => panic!(msg),
        };

        assert_eq!(
            parse(tokens),
            Ok(seq![
                Body::Guard(
                    selector!(m rmatch!("thing")),
                    seq![Body::Bare(Statement::Subst(Box::new(Regex::new("that").unwrap()), quote!(s"other")))]
                )
            ])
        );
    }

    #[test]
    fn parse_statement_gsubst() {
        let tokens = match lex("/thing/ { gsubst /that/, 'other' }") {
            Ok(tokens) => tokens,
            Err(msg) => panic!(msg),
        };

        assert_eq!(
            parse(tokens),
            Ok(seq![
                Body::Guard(
                    selector!(m rmatch!("thing")),
                    seq![Body::Bare(Statement::Gsubst(Box::new(Regex::new("that").unwrap()), quote!(s"other")))]
                )
            ])
        );
    }

    #[test]
    fn parse_statement_read() {
        let tokens = match lex("/thing/ { read 'somefile.txt' }") {
            Ok(tokens) => tokens,
            Err(msg) => panic!(msg),
        };

        assert_eq!(
            parse(tokens),
            Ok(seq![
                Body::Guard(
                    selector!(m rmatch!("thing")),
                    seq![Body::Bare(Statement::Read(quote!(s"somefile.txt")))]
                )
            ])
        );
    }

    #[test]
    fn parse_statement_write() {
        let tokens = match lex("/thing/ { write 'somefile.txt' }") {
            Ok(tokens) => tokens,
            Err(msg) => panic!(msg),
        };

        assert_eq!(
            parse(tokens),
            Ok(seq![
                Body::Guard(
                    selector!(m rmatch!("thing")),
                    seq![Body::Bare(Statement::Write(quote!(s"somefile.txt")))]
                )
            ])
        );
    }

    #[test]
    fn parse_statement_execute() {
        let tokens = match lex("/thing/ { exec \"echo ${_}\" }") {
            Ok(tokens) => tokens,
            Err(msg) => panic!(msg),
        };

        assert_eq!(
            parse(tokens),
            Ok(seq![
                Body::Guard(
                    selector!(m rmatch!("thing")),
                    seq![Body::Bare(Statement::Exec(quote!("echo ${_}")))]
                )
            ])
        );
    }

    #[test]
    fn parse_single() {
        let tokens = match lex("/thing/ exec \"echo ${_}\"") {
            Ok(tokens) => tokens,
            Err(msg) => panic!(msg),
        };

        assert_eq!(
            parse(tokens),
            Ok(seq![
                Body::Single(
                    selector!(m rmatch!("thing")),
                    Statement::Exec(quote!("echo ${_}"))
                )
            ])
        );
    }
}
