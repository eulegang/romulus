///! Utilities for the lex function.
///! Handles low level character gathering
use std::iter::Peekable;
use std::ops::RangeInclusive;

/// Gathers characters while the characters are in
/// the accepted set of characters
#[inline]
pub fn chomp_range<T: Iterator<Item = (usize, char)>>(
    iter: &mut Peekable<T>,
    accept: RangeInclusive<char>,
) -> Vec<char> {
    let mut accepted = Vec::new();

    while let Some((_, ch)) = &mut iter.peek() {
        if accept.contains(ch) {
            accepted.push(*ch);
            iter.next();
        } else {
            break;
        }
    }

    accepted
}

/// Gathers characters while the current char is in chars or
/// in one of the ranges in accepts
#[inline]
pub fn chomp_multi<T: Iterator<Item = (usize, char)>>(
    iter: &mut Peekable<T>,
    chars: &[char],
    accepts: &[RangeInclusive<char>],
) -> Vec<char> {
    let mut accepted = Vec::new();

    'base: while let Some((_, ch)) = &mut iter.peek() {
        let owned = *ch;

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

/// gathers characters while the current char is in the accept set
#[inline]
pub fn chomp_set<T: Iterator<Item = (usize, char)>>(
    iter: &mut Peekable<T>,
    accept: &[char],
) -> Vec<char> {
    let mut accepted = Vec::new();

    while let Some((_, ch)) = &mut iter.peek() {
        let owned: char = *ch;

        if accept.contains(&owned) {
            accepted.push(owned);
            iter.next();
        } else {
            break;
        }
    }

    accepted
}

/// gather characters until the terminating character
/// if this range of characters is meant to be evaluated with $
/// then \$ is kept together to distignuish it from actual evaultion
/// at runtime.
#[inline]
pub fn chomp_until_escaped<T: Iterator<Item = (usize, char)>>(
    iter: &mut Peekable<T>,
    terminator: char,
    evaluates: bool,
) -> Result<Vec<char>, String> {
    let mut accepted: Vec<char> = Vec::new();

    while let Some((_, ch)) = &mut iter.peek() {
        let owned: char = *ch;

        if owned == '\\' {
            iter.next();
            match iter.next() {
                Some((_, 'n')) => accepted.push('\n'),
                Some((_, '\\')) => accepted.push('\\'),
                Some((_, 't')) => accepted.push('\t'),
                Some((_, 'r')) => accepted.push('\r'),
                Some((_, '$')) if evaluates => {
                    accepted.push('\\');
                    accepted.push('$')
                }
                Some((_, escaped)) if escaped == terminator => accepted.push(escaped),
                Some((_, escaped)) => return Err(format!("cannot escape {}", escaped)),
                None => return Err(format!("found EOF when searching for {}", &terminator)),
            }
        } else if owned != terminator {
            accepted.push(owned);
            iter.next();
        } else {
            break;
        }
    }

    Ok(accepted)
}

/// Gather characters until a character in the accept set is found
#[inline]
pub fn chomp_until_set<T: Iterator<Item = (usize, char)>>(
    iter: &mut Peekable<T>,
    accept: &[char],
) -> Vec<char> {
    let mut accepted: Vec<char> = Vec::new();

    while let Some((_, ch)) = &mut iter.peek() {
        let owned: char = *ch;

        if !accept.contains(&owned) {
            accepted.push(owned);
            iter.next();
        } else {
            break;
        }
    }

    accepted
}

/// Evaulate the character buffer as a number
#[inline]
pub fn get_number(vec: Vec<char>) -> i64 {
    let mut buffer = 0;
    for ch in vec {
        let digit = ch.to_string().parse::<i64>().unwrap();

        buffer = buffer * 10 + digit;
    }

    buffer
}
