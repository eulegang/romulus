//! A module which detects what features romulus was compiled with.
//!
//! Used for runtime introspection rather then
//! conditional compilation.

macro_rules! feature {
    ($feat: expr) => {
        (cfg!(feature = $feat), $feat.to_string())
    };
}

/// Gives a record of which features romulus was installed with
pub fn features() -> Vec<(bool, String)> {
    vec![feature!("color"), feature!("envvar"), feature!("stdin-tty")]
}
