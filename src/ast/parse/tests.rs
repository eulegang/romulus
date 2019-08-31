use super::*;
use crate::lex::lex;

macro_rules! seq {
    (tl $($ast: expr),*) => {
        {
            let mut subnodes = Vec::new();

            $(
                subnodes.push($ast);
            )*

            Seq { subnodes, toplevel: true }
        }
    };

    ($($ast: expr),*) => {
        {
            let mut subnodes = Vec::new();

            $(
                subnodes.push($ast);
            )*

            Seq { subnodes, toplevel: false }
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
    (!$ast: expr) => { Selector::Negate(Box::new($ast)) };
    (a$lh : expr, $rh : expr) => { Selector::Conjunction(Box::new($lh), Box::new($rh)) };
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
        Ok(seq![tl Body::Guard(
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
        Ok(seq![tl Body::Bare(Statement::Print(quote![s"found it"]))])
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
        Ok(seq![tl Body::Guard(
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
        Ok(seq![tl Body::Guard(
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
        Ok(seq![tl Body::Guard(
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
        Ok(seq![tl
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
        Ok(seq![tl
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
        Ok(seq![tl
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
        Ok(seq![tl
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
        Ok(seq![tl
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
        Ok(seq![tl
            Body::Guard(
                selector!(m rmatch!("thing")),
                seq![Body::Bare(Statement::Exec(quote!("echo ${_}")))]
            )
        ])
    );
}

#[test]
fn parse_statement_append() {
    let tokens = match lex("/backup/ { append '.bak' }") {
        Ok(tokens) => tokens,
        Err(msg) => panic!(msg),
    };

    assert_eq!(
        parse(tokens),
        Ok(seq![tl
            Body::Guard(
                selector!(m rmatch!("backup")),
                seq![Body::Bare(Statement::Append(quote!(s".bak")))]
            )
        ])
    );
}

#[test]
fn parse_statement_set() {
    let tokens = match lex("/backup/ { set '.bak' }") {
        Ok(tokens) => tokens,
        Err(msg) => panic!(msg),
    };

    assert_eq!(
        parse(tokens),
        Ok(seq![tl
            Body::Guard(
                selector!(m rmatch!("backup")),
                seq![Body::Bare(Statement::Set(quote!(s".bak")))]
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
        Ok(seq![tl
            Body::Single(
                selector!(m rmatch!("thing")),
                Statement::Exec(quote!("echo ${_}"))
            )
        ])
    );
}

#[test]
fn parse_negation() {
    let tokens = lex("!/thing/ exec \"echo ${_}\"").unwrap();

    assert_eq!(
        parse(tokens),
        Ok(seq![tl
            Body::Single(
                selector!(! selector!(m rmatch!("thing"))),
                Statement::Exec(quote!("echo ${_}"))
            )
        ])
    )
}

#[test]
fn parse_conjunction() {
    let tokens = lex("/thing/ & /other/ print _").unwrap();

    assert_eq!(
        parse(tokens),
        Ok(seq![tl
            Body::Single(
                selector!(a selector!(m rmatch!("thing")), selector!(m rmatch!("other"))),
                Statement::Print(Expression::Identifier("_".to_string()))
            )
        ])
    );
}
