use super::Valuable;
use super::*;
use crate::ast::*;
use ansi_term::Colour::Red;
use regex::Regex;
use Event::*;

macro_rules! error {
    ($format: expr, $($args: expr),*) => {
        {
            eprint!("{}{}", color!(Red, format!($format, $($args),*)), nl!());
            return
        }
    }
}

pub fn print(expr: &Expression, env: &mut Environment) {
    env.print(&mut format!("{}{}", expr.to_value(env), nl!()).as_bytes());
}

pub fn quit(env: &mut Environment) {
    env.quit()
}

pub fn subst(regex: &Regex, expr: &Expression, env: &mut Environment) {
    env.replace_line(|env, line| {
        regex
            .replace(&line, |caps: &Captures| {
                env.eval(Scope::from_captures(regex, caps), expr)
            })
            .into_owned()
    });
}

pub fn gsubst(regex: &Regex, expr: &Expression, env: &mut Environment) {
    env.replace_line(|env, line| {
        regex
            .replace_all(&line, |caps: &Captures| {
                env.eval(Scope::from_captures(regex, caps), expr)
            })
            .into_owned()
    });
}

pub fn read(expr: &Expression, env: &mut Environment) {
    let mut file = match std::fs::File::open(expr.to_value(env)) {
        Ok(f) => f,
        Err(msg) => error!("Error open file {}", msg),
    };

    env.print(&mut file);
}

pub fn write(expr: &Expression, env: &mut Environment) {
    if let Line(line) = &env.event {
        let mut file = match std::fs::OpenOptions::new()
            .write(true)
            .append(true)
            .create(true)
            .open(expr.to_value(env))
        {
            Ok(f) => f,
            Err(msg) => error!("Error opening file {}", msg),
        };

        match write!(file, "{}{}", line, nl!()) {
            Ok(_) => (),
            Err(msg) => error!("Error writing to file {}", msg),
        }
    }
}

pub fn exec(expr: &Expression, env: &mut Environment) {
    match shell(&expr.to_value(env)) {
        Err(msg) => {
            let message = format!("unable to execute: {}", msg);
            eprint!("{}{}", color!(Red, message), nl!());
        }

        Ok(child) => {
            env.print(&mut child.stdout.unwrap());
        }
    }
}

pub fn append(expr: &Expression, env: &mut Environment) {
    env.replace_line(|env, line| format!("{}{}", line, expr.to_value(&env)));
}

pub fn set(expr: &Expression, env: &mut Environment) {
    env.replace_line(|env, _| expr.to_value(env))
}

#[cfg(not(target_os = "windows"))]
fn shell(cmd: &str) -> Result<std::process::Child, std::io::Error> {
    Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .stdout(Stdio::piped())
        .spawn()
}
#[cfg(target_os = "windows")]
fn shell(cmd: &str) -> Result<std::process::Child, std::io::Error> {
    Command::new("cmd")
        .arg("/C")
        .arg(cmd)
        .stdout(Stdio::piped())
        .spawn()
}
