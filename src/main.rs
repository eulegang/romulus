
extern crate clap;

mod lang;

use clap::{App, Arg};

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

    println!("Hello, world!");
}
