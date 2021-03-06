//! Romulus is a text processing language similar to sed
//!
//! Here is an example program which process the output of ifconfig
//! ```text
//! /^(?P<inter>[a-zA-Z0-9]+): /,/^[a-zA-Z0-9]+:/ {
//!   /inet (?P<ip>[0-9]{1,3}(\.[0-9]{1,3}){3})/ {
//!     print("${inter}: ${ip}")
//!   }
//!
//!   /inet6 (?P<ip>[a-fA-F0-9]{0,4}(:[a-fA-F0-9]{0,4}){0,8})/ {
//!     print("${inter}: ${ip}")
//!   }
//! }
//! ```
//!

#![deny(missing_docs)]

/// A macro that expands to the proper line ending character sequence
/// for operating system
#[macro_export]
macro_rules! nl {
    () => {
        if cfg!(not(target_os = "windows")) {
            "\n"
        } else {
            "\r\n"
        }
    };
}

/// A macro that expands to a colored output if romulus is compiled
/// with color support
#[macro_export]
macro_rules! color {
    ($color: expr, $msg: expr) => {
        if cfg!(feature = "color") {
            $color.paint($msg.to_string()).to_string()
        } else {
            $msg.to_string()
        }
    };
}

#[macro_use]
extern crate lazy_static;

mod interpreter;

mod ast;
mod features;
mod lex;
mod lint;
mod runtime;

pub use features::*;
pub use interpreter::{Builder, Interpreter};
