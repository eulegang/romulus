///! Utilities for the lex function.
///! Handles low level character gathering
use std::iter::Peekable;
use std::ops::RangeInclusive;

pub trait Chomper {
    fn accept(&self, ch: char) -> bool;
}

pub fn chomp<Iter>(chomper: &dyn Chomper, peekable: &mut Peekable<Iter>) -> usize
where
    Iter: Iterator<Item = (usize, char)>,
{
    let mut cur = 0;
    while let Some((idx, ch)) = peekable.peek() {
        cur = *idx;

        if !chomper.accept(*ch) {
            return cur;
        }

        peekable.next();
    }

    cur + 1
}

pub fn chomp_until<Iter>(chomper: &dyn Chomper, peekable: &mut Peekable<Iter>) -> usize
where
    Iter: Iterator<Item = (usize, char)>,
{
    let mut cur = 0;
    while let Some((idx, ch)) = peekable.peek() {
        cur = *idx;

        if chomper.accept(*ch) {
            return cur;
        }

        peekable.next();
    }

    cur + 1
}

pub fn chomp_str<Iter>(chomper: &dyn Chomper, peekable: &mut Peekable<Iter>) -> String
where
    Iter: Iterator<Item = (usize, char)>,
{
    let mut accepted = String::new();

    while let Some((_, ch)) = peekable.peek() {
        if chomper.accept(*ch) {
            accepted.push(*ch);
            peekable.next();
        } else {
            break;
        }
    }

    accepted
}

pub struct Multi<'a, C: Chomper>(pub &'a [&'a C]);

impl Chomper for RangeInclusive<char> {
    fn accept(&self, ch: char) -> bool {
        self.contains(&ch)
    }
}

impl Chomper for [char] {
    fn accept(&self, ch: char) -> bool {
        self.contains(&ch)
    }
}

impl Chomper for &[char; 1] {
    fn accept(&self, ch: char) -> bool {
        self.contains(&ch)
    }
}

impl Chomper for &[char; 2] {
    fn accept(&self, ch: char) -> bool {
        self.contains(&ch)
    }
}

impl Chomper for &[char; 3] {
    fn accept(&self, ch: char) -> bool {
        self.contains(&ch)
    }
}

impl<C: Chomper> Chomper for Multi<'_, C> {
    fn accept(&self, ch: char) -> bool {
        for sub in self.0 {
            if sub.accept(ch) {
                return true;
            }
        }

        false
    }
}

impl<A: Chomper, B: Chomper> Chomper for (A, B) {
    fn accept(&self, ch: char) -> bool {
        self.0.accept(ch) || self.1.accept(ch)
    }
}

/// gather characters until the terminating character
/// if this range of characters is meant to be evaluated with $
/// then \$ is kept together to distignuish it from actual evaultion
/// at runtime.
#[inline]
pub fn chomp_until_escaped<T: Iterator<Item = (usize, char)>>(
    iter: &mut Peekable<T>,
    terminator: char,
    escapable: &[char], //evaluates: bool,
) -> Result<String, String> {
    let mut accepted = String::new();

    while let Some((_, ch)) = &mut iter.peek() {
        let owned: char = *ch;

        if owned == '\\' {
            iter.next();
            match iter.next() {
                Some((_, escaped)) if escaped == terminator => accepted.push(escaped),
                Some((_, escaped)) if escapable.contains(&escaped) => {
                    accepted.push('\\');
                    accepted.push(escaped);
                }

                Some((_, 'n')) => accepted.push('\n'),
                Some((_, 't')) => accepted.push('\t'),
                Some((_, 'r')) => accepted.push('\r'),
                Some((_, '\\')) => accepted.push('\\'),
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

/// Evaulate the character buffer as a number
#[inline]
pub fn get_number(vec: &str) -> i64 {
    let mut buffer = 0;
    for ch in vec.chars() {
        let digit = ch.to_string().parse::<i64>().unwrap();

        buffer = buffer * 10 + digit;
    }

    buffer
}
