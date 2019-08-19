#[macro_use]
extern crate clap;

extern crate tempfile;
extern crate atty;
extern crate ansi_term;

#[macro_use(nl)]
extern crate romulus;

extern crate regex;

use clap::{App, Arg, ArgGroup, ArgMatches};
use regex::Regex;
use romulus::Interpreter;
use std::fmt::Display;
use std::fs::{self, File};
use std::io::{stdin, stdout, BufReader, Write};
use std::process;
use ansi_term::Colour::*;

macro_rules! color {
    ($color: expr, $msg: expr) => {
        if cfg!(feature = "color") {
            $color.paint($msg.to_string()).to_string()
        } else {
            $msg.to_string()
        }
    }
}

fn main() {
    let matches = App::new("romulus")
        .version(crate_version!())
        .about("a text stream editor")
        .arg(
            Arg::with_name("expr")
                .short("e")
                .long("expr")
                .takes_value(true)
                .help("romulus expression"),
        )
        .arg(
            Arg::with_name("file")
                .short("f")
                .long("file")
                .takes_value(true)
                .help("file with romulus program"),
        )
        .arg(
            Arg::with_name("output")
                .short("o")
                .long("out")
                .takes_value(true)
                .help("output file"),
        )
        .arg(
            Arg::with_name("inplace")
                .short("i")
                .long("inplace")
                .takes_value(true)
                .requires("inputs")
                .help("inplace replacement backup extension"),
        )
        .arg(
            Arg::with_name("features")
                .long("features")
                .help("prints which features are enabled")
        )
        .arg(
            Arg::with_name("sep")
                .short("s")
                .long("sep")
                .env("RSEP")
                .takes_value(true)
                .default_value(" +")
                .help("sepeartes patterns in a line"),
        )
        .arg(
            Arg::with_name("lint")
                .short("l")
                .long("lint")
                .env("RLINT")
                .takes_value(true)
                .possible_values(&["off", "warn", "strict"])
                .default_value("warn")
                .help("selects the behavior of linting")
        )
        .group(
            ArgGroup::with_name("program")
                .args(&["file", "expr", "features"])
                .required(true),
        )
        .group(ArgGroup::with_name("output_flow").args(&["output", "inplace"]))
        .arg(Arg::with_name("inputs").min_values(1))
        .get_matches();

    if matches.is_present("features") {
        print_features();
        process::exit(0);
    }

    let interpreter = create_interpreter(&matches);

    lint(&interpreter, &matches);

    if let Some(ext) = matches.value_of("inplace") {
        process_inplace(interpreter, ext, &mut matches.values_of("inputs").unwrap())
    } else {
        process_streams(interpreter, &matches);
    }
}

fn create_interpreter(matches: &ArgMatches) -> Interpreter {
    let sep = match Regex::new(matches.value_of("sep").unwrap()) {
        Ok(regex) => regex,
        Err(msg) => {
            eprint!("Error parsing sep: {}{}", msg, nl!());
            process::exit(1);
        }
    };

    if let Some(expr) = matches.value_of("expr") {
        return ok_or_exit(Interpreter::new(expr, sep));
    }

    if let Some(filename) = matches.value_of("file") {
        return ok_or_exit(Interpreter::file(filename, sep));
    }

    unreachable!()
}

fn process_inplace<'a, I: Iterator<Item = &'a str>>(
    interpreter: Interpreter,
    ext: &str,
    inputs: &'a mut I,
) {
    for input in inputs {
        let fin = match File::open(&input) {
            Ok(f) => f,
            Err(err) => {
                eprint!("unable to read file '{}': {}{}", input, err, nl!());
                process::exit(1);
            }
        };

        let mut fout = match tempfile::NamedTempFile::new() {
            Ok(f) => f,
            Err(err) => {
                eprint!("unable to create temp file {}{}", err, nl!());
                process::exit(1);
            }
        };

        interpreter.process(&mut BufReader::new(fin), &mut fout);

        if ext != "" {
            if let Err(err) = fs::rename(&input, format!("{}.{}", input, ext)) {
                drop(fout);
                eprint!("unable to rename {}.{} -> {}: {}{}", input, ext, input, err, nl!());
                process::exit(1);
            }
        }

        if let Err(err) = fout.persist(input) {
            eprint!("unable to replace {}: {}{}", input, err, nl!());
            process::exit(1);
        };
    }
}

fn process_streams(interpreter: Interpreter, matches: &ArgMatches) {
    let mut output: Box<dyn Write> = match matches.value_of("output") {
        Some(filename) => match File::create(filename) {
            Ok(f) => Box::new(f),
            Err(_) => {
                eprint!("Unable to create {}{}", filename, nl!());
                process::exit(1);
            }
        },

        None => Box::new(stdout()),
    };

    if let Some(inputs) = matches.values_of("inputs") {
        for input in inputs {
            let file = match File::open(input) {
                Ok(f) => f,
                Err(_) => {
                    eprint!("Unable to read {}{}", input, nl!());
                    process::exit(1);
                }
            };

            interpreter.process(&mut BufReader::new(file), &mut output);
        }
    } else {
        if cfg!(not(feature = "stdin-tty")) && atty::is(atty::Stream::Stdin) {
            eprint!("{}{}", color!(Red, "Stdin is a tty refusing to process"), nl!());
            process::exit(1)
        }

        let sin = stdin();
        let mut sin_lock = sin.lock();

        interpreter.process(&mut sin_lock, &mut output);
    }
}

#[inline]
fn ok_or_exit<T, E: Display>(result: Result<T, E>) -> T {
    match result {
        Ok(t) => t,
        Err(msg) => {
            eprint!("{}{}", msg, nl!());
            process::exit(1);
        }
    }
}

fn lint(interpreter: &Interpreter, matches: &ArgMatches) {
    match matches.value_of("lint") {
        Some("off") => (),
        Some("warn") => {
            for msg in interpreter.lint() {
                eprint!("{}{}", color!(Yellow, msg), nl!());
            }
        }

        Some("strict") => {
            let msgs = interpreter.lint();

            for msg in &msgs {
                eprint!("{}{}", color!(Red, &msg), nl!());
            }

            if !msgs.is_empty() {
                process::exit(1)
            }
        }

        _ => unreachable!(),
    }
}

fn print_features() {
    for (enabled, feature) in romulus::features::features() {
        let repr = if enabled { "+" }  else { "-" };
        let color = if enabled { Green } else { Red };

        print!("{}{}{}", color!(color, repr), feature, nl!());
    }
}
