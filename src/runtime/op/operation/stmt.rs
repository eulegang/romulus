use super::*;
use crate::ast::*;
use regex::Regex;
use Event::*;

pub fn print(expr: &Expression, env: &mut Environment) {
    let _ = write!(env.out, "{}{}", expr.to_value(env), nl!());
}

pub fn quit(env: &mut Environment) {
    env.quit = true
}

pub fn subst(regex: &Regex, expr: &Expression, env: &mut Environment) {
    let line = match &env.event {
        Line(line) => line.to_string(),
        _ => return
    };

    env.event = Line(regex.replace(&line, |caps: &Captures|  {
        env.push(Scope::from_captures(regex, caps));
        let result = expr.to_value(env);
        env.pop();
        result
    }).into_owned());
}

pub fn gsubst(regex: &Regex, expr: &Expression, env: &mut Environment) {
    let line = match &env.event {
        Line(line) => line.to_string(),
        _ => return
    };

    env.event = Line(regex.replace_all(&line, |caps: &Captures| { 
        env.push(Scope::from_captures(regex, caps));
        let result = expr.to_value(env);
        env.pop();

        result
    }).into_owned());
}

pub fn read(expr: &Expression, env: &mut Environment) {
    let mut file = match std::fs::File::open(expr.to_value(env)) {
        Ok(f) => f,
        Err(msg) => {
            eprint!("Error open file {}{}", msg, nl!());
            return
        }
    };

    match copy(&mut file, env.out) {
        Ok(_) => (),
        Err(msg) => {
            eprint!("Error cating file {}{}", msg, nl!());
        }
    }
}

pub fn write(expr: &Expression, env: &mut Environment) {
    if let Line(line) = &env.event {
        let mut file = match std::fs::OpenOptions::new()
            .write(true)
            .append(true)
            .create(true).open(expr.to_value(env)) {
                Ok(f) => f,
                Err(msg) => {
                    eprint!("Error oppening file {}{}", msg, nl!());
                    return;
                }
            };

        match write!(file, "{}{}", line, nl!()) {
            Ok(_) => (),
            Err(msg) => {
                eprint!("Error writing to file {}{}", msg, nl!());
                return;
            }
        }
    }
}

pub fn exec(expr: &Expression, env: &mut Environment) {
    match shell(&expr.to_value(env)) {
        Err(msg) => {
            eprint!("unable to execute: {}{}", msg, nl!());
        }

        Ok(child) => {
            let _ = copy(&mut child.stdout.unwrap(), env.out);
        }
    };
}

pub fn append(expr: &Expression, env: &mut Environment) {
    if let Line(line) = &env.event {
        env.event = Line(format!("{}{}", line, expr.to_value(&env)));
    }
}

pub fn set(expr: &Expression, env: &mut Environment) {
    if let Line(_) = &env.event {
        env.event = Line(expr.to_value(&env));
    }
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
