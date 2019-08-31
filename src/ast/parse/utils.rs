use super::*;

#[inline]
pub(super) fn parse_until<T: Parsable>(
    token: Token,
    tokens: &[Token],
    pos: &mut usize,
) -> Result<Vec<T>, String> {
    let mut subnodes = Vec::new();

    while Some(&token) != tokens.get(*pos) {
        subnodes.push(<T>::parse_mut(&tokens, pos)?);
    }

    expect_token(token, tokens, pos)?;

    Ok(subnodes)
}

pub(super) fn expect_token(token: Token, tokens: &[Token], pos: &mut usize) -> Result<(), String> {
    match tokens.get(*pos) {
        Some(t) if t == &token => {
            *pos += 1;
            Ok(())
        }
        Some(t) => Err(format!("expected {:?} but recieved {:?}", token, t)),
        None => Err("unexpected EOF".to_string()),
    }
}
