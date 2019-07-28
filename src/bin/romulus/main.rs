
#[macro_use]
extern crate clap;

extern crate tempfile;

extern crate romulus;

use clap::{App, Arg, ArgGroup, ArgMatches};
use std::fs::{self, File};
use std::io::{stdin, stdout, BufReader, Write};
use std::process;
use romulus::Interpreter;
use std::fmt::Display;

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
                .help("inplace replacement backup extension")
        )
        .group(
            ArgGroup::with_name("program")
                .args(&["file", "expr"])
                .required(true),
        )
        .group(
            ArgGroup::with_name("output_flow")
                .args(&["output", "inplace"])
        )
        .arg(Arg::with_name("inputs").min_values(1))
        .get_matches();

    let interpreter = create_interpreter(&matches);
    if let Some(ext) = matches.value_of("inplace") {
        process_inplace(interpreter, ext, &mut matches.values_of("inputs").unwrap())
    } else {
        process_streams(interpreter, &matches);
    }
}

fn create_interpreter(matches: &ArgMatches) -> Interpreter {
    if let Some(expr) = matches.value_of("expr") {
        return ok_or_exit(Interpreter::new(expr));
    }

    if let Some(filename) = matches.value_of("file") {
        return ok_or_exit(Interpreter::file(filename))
    }

    unreachable!()
}

fn process_inplace<'a, I: Iterator<Item=&'a str>>(interpreter: Interpreter, ext: &str, inputs: &'a mut I) {
    for input in inputs {
        let fin = match File::open(&input) {
            Ok(f) => f,
            Err(err) => {
                eprintln!("unable to read file '{}': {}", input, err);
                process::exit(1);
            }
        };

        let mut fout = match tempfile::NamedTempFile::new() {
            Ok(f) => f,
            Err(err) => {
                eprintln!("unable to create temp file {}", err);
                process::exit(1);
            }
        };

        interpreter.process(&mut BufReader::new(fin), &mut fout);

        if ext != "" {
            if let Err(err) = fs::rename(&input, format!("{}.{}", input, ext)) {
                drop(fout);
                eprintln!("unable to rename {}.{} -> {}: {}", input, ext, input, err);
                process::exit(1);
            }
        }

        if let Err(err) = fout.persist(input) {
            eprintln!("unable to replace {}: {}", input, err);
            process::exit(1);
        };
    }
}

fn process_streams(interpreter: Interpreter, matches: &ArgMatches) {
    let mut output: Box<dyn Write> = match matches.value_of("output") {
        Some(filename) => match File::create(filename) {
            Ok(f) => Box::new(f),
            Err(_) => {
                eprintln!("Unable to create {}", filename);
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
                    eprintln!("Unable to read {}", input);
                    process::exit(1);
                }
            };

            interpreter.process(&mut BufReader::new(file), &mut output);
        }
    } else {
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
            eprintln!("{}", msg);
            process::exit(1);
        }
    }
}

