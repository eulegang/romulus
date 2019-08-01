use super::{Environment, Value};
use std::io::Write;

pub(crate) fn print(env: &mut Environment, args: &[Value]) {
    if args.is_empty() {
        let _ = write!(env.out, "{}", &env.line);
    } else {
        for arg in args {
            let _ = write!(env, "{}", arg);
        }
    }

    let _ = writeln!(env);
}
