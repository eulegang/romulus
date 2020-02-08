#[macro_use]
extern crate clap;

extern crate ansi_term;
extern crate atty;
extern crate tempfile;

#[macro_use(nl, color)]
extern crate romulus;

extern crate regex;

use ansi_term::Colour::*;
use clap::{App, Arg, ArgGroup, ArgMatches};
use regex::Regex;
use romulus::Interpreter;
use std::fs::{self, File};
use std::io::{stdin, stdout, BufReader, Write};
use std::process;

macro_rules! error {
    ($format: expr, $($args: expr),*) => {
        {
            eprint!("{}{}", color!(Red, format!($format, $($args),*)), nl!());
            std::process::exit(1);
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
                .help("prints which features are enabled"),
        )
        .arg(
            Arg::with_name("sep")
                .short("s")
                .long("sep")
                .env("RSEP")
                .takes_value(true)
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
                .help("selects the behavior of linting"),
        )
        .arg(
            Arg::with_name("explicit")
                .short("E")
                .long("explicit")
                .takes_value(false)
                .help("disable implicit line printing"),
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
    let mut builder = Interpreter::builder();

    if let Some(sep) = matches.value_of("sep") {
        match Regex::new(sep) {
            Ok(regex) => {
                let _ = builder.sep(regex);
            }
            Err(msg) => error!("Error parsing sep: {}", msg),
        }
    }

    if let Some(expr) = matches.value_of("expr") {
        builder.expression(expr.to_string());
    }

    if matches.is_present("explicit") {
        builder.print(false);
    }

    if let Some(filename) = matches.value_of("file") {
        builder.filename(filename.to_string());
    }

    match builder.build() {
        Ok(interpreter) => interpreter,
        Err(msg) => error!("Failed to build interpreter: {}", msg),
    }
}

fn process_inplace<'a, I: Iterator<Item = &'a str>>(
    interpreter: Interpreter,
    ext: &str,
    inputs: &'a mut I,
) {
    for input in inputs {
        let fin = match File::open(&input) {
            Ok(f) => f,
            Err(err) => error!("unable to read file '{}': {}", input, err),
        };

        let mut fout = match tempfile::NamedTempFile::new() {
            Ok(f) => f,
            Err(err) => error!("unable to create temp file {}", err),
        };

        interpreter.process(&mut BufReader::new(fin), &mut fout);

        if ext != "" {
            if let Err(err) = fs::rename(&input, format!("{}.{}", input, ext)) {
                drop(fout);
                error!("unable to rename {}.{} -> {}: {}", input, ext, input, err);
            }
        }

        if let Err(err) = fout.persist(input) {
            if err.error.kind() == std::io::ErrorKind::Other { // try to cp it manually
                if let Err(err) = std::fs::copy(err.file.path(), input) {
                    error!("unable to replace {}: {}", input, err);
                }
            } else {
                error!("unable to replace {}: {}", input, err);
            }
        };
    }
}

fn process_streams(interpreter: Interpreter, matches: &ArgMatches) {
    let mut output: Box<dyn Write> = match matches.value_of("output") {
        Some(filename) => match File::create(filename) {
            Ok(f) => Box::new(f),
            Err(_) => error!("unable to create {}", filename),
        },

        None => Box::new(stdout()),
    };

    if let Some(inputs) = matches.values_of("inputs") {
        for input in inputs {
            let file = match File::open(input) {
                Ok(f) => f,
                Err(_) => error!("Unable to read {}", input),
            };

            interpreter.process(&mut BufReader::new(file), &mut output);
        }
    } else {
        if cfg!(not(feature = "stdin-tty")) && atty::is(atty::Stream::Stdin) {
            error!("{}", "Stdin is a tty, refusing to process");
        }

        let sin = stdin();
        let mut sin_lock = sin.lock();

        interpreter.process(&mut sin_lock, &mut output);
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
    for (enabled, feature) in romulus::features() {
        let repr = if enabled { "+" } else { "-" };
        let color = if enabled { Green } else { Red };

        print!("{}{}{}", color!(color, repr), feature, nl!());
    }
}
