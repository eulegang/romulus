//! Romulus is a text processing language similar to sed
//!
//! Here is an example program which process the output of ifconfig
//! ```text
//! /^(?P<inter>[a-zA-Z0-9]+): /,/^[a-zA-Z0-9]+:/ {
//!	  /inet (?P<ip>[0-9]{1,3}(\.[0-9]{1,3}){3})/ {
//!     print("${inter}: ${ip}")
//!	  }
//!
//!	  /inet6 (?P<ip>[a-fA-F0-9]{0,4}(:[a-fA-F0-9]{0,4}){0,8})/ {
//!     print("${inter}: ${ip}")
//!	  }
//! }
//! ```
//!

#![allow(clippy::new_without_default)]

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
    }
}

#[macro_use]
extern crate lazy_static;

mod interpreter;

pub mod ast;
pub mod lex;
pub mod runtime;
pub mod lint;
pub mod features;

pub use interpreter::Interpreter;
