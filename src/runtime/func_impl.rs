use super::{Environment, Value, Event};
use std::io::Write;

pub(crate) fn print(env: &mut Environment, args: &[Value]) {
    if args.is_empty() {
        if let Event::Line(line) = &env.event {
            let _ = write!(env.out, "{}", line);
            let _ = writeln!(env);
        }
    } else {
        for arg in args {
            let _ = write!(env, "{}", arg);
        }
        let _ = writeln!(env);
    }
}
