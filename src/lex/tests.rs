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
        Token::Comment("some comment"),
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

#[test]
fn escape() {
    let tokens = vec![Token::String("\\\n\"".to_string(), true)];

    assert_eq!(lex("\"\\\\\\n\\\"\""), Ok(tokens))
}
