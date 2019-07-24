
#[macro_use]
extern crate lazy_static;
extern crate clap;

mod lang;

use clap::{App, Arg};
use std::process;
use std::fs::File;
use std::io::{Read,stdin,stdout};

fn main() {
    let matches = App::new("romulus")
        .version("0.1.0")
        .about("a text stream editor")
        .arg(Arg::with_name("expr")
             .short("e")
             .long("expr")
             .takes_value(true)
             .help("romulus expression"))
        .arg(Arg::with_name("file")
             .short("f")
             .long("file")
             .takes_value(true)
             .help("file with romulus program"))
        .get_matches();

    let interpreter = match (matches.value_of("expr"), matches.value_of("file")) {
        (Some(expr), None) => interpreter_expr(expr),
        (None, Some(filename)) => interpreter_file(filename),
        (None, None) => {
            eprintln!("Must specify an expression or a file");
            process::exit(1);
        }
        (Some(_), Some(_)) => {
            eprintln!("Must specify an expression or a file not both");
            process::exit(1);
        }
    };

    let sin = stdin();
    let mut sin_lock = sin.lock();
    let mut sout = stdout();
    interpreter.process(&mut sin_lock, &mut sout);
}

fn interpreter_expr(expr: &str) -> lang::Interpreter {
    match lang::Interpreter::new(expr) {
        Ok(int) => int,
        Err(msg) => {
            eprintln!("{}", msg);
            process::exit(1);
        }
    }
}

fn interpreter_file(path: &str) -> lang::Interpreter {
    let mut file = match File::open(path)  {
        Ok(f) => f,
        Err(err) => {
            eprintln!("{}", err);
            process::exit(1);
        }
    };

    let mut buf = String::new();
    if let Err(err) = file.read_to_string(&mut buf) {
        eprintln!("{}", err);
        process::exit(1);
    }

    match lang::Interpreter::new(&buf) {
        Ok(f) => f,
        Err(msg) => {
            eprintln!("{}", msg);
            process::exit(1);
        }
    }
}
