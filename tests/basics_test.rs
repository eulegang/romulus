extern crate romulus;

use romulus::Interpreter;
use romulus::runtime::FunctionRegistry;

macro_rules! run_interpreter {
    ($prog: expr, $input: expr) => {{
        let interpreter = Interpreter::new($prog, FunctionRegistry::default()).unwrap();
        let mut out = Vec::new();
        let mut sin = $input.as_bytes();

        interpreter.process(&mut sin, &mut out);
        String::from_utf8(out).unwrap()
    }};
}

#[test]
fn basic() {
    assert_eq!(
        run_interpreter!(
            "/needle/ { print('found it') }",
            "hay\nhay\nhey\nneedle\nhay"
        ),
        "found it\n".to_string()
    );
}

#[test]
fn range() {
    assert_eq!(
        run_interpreter!(
            "/third/,/fifth/ { print() }",
            "first\nsecond\nthird\nfourth\nfifth\nsexth\nseventh\nninth\ntenth\n"
        ),
        "third\nfourth\n"
    );
}

#[test]
fn capture() {
    assert_eq!(
        run_interpreter!(
            "1 { print('name,type') }\n/pokemon \"(?P<name>.*)\"/,/}/ { /type *= *\"(?P<type>.*)\"/ { print(name, ',', type) } }",
            "pokemon \"Haunter\" {\ntype = \"Ghost\"\n}\npokemon \"Noctowl\" {\ntype = \"Flying\"\n}\n"
        ),
        "name,type\nHaunter,Ghost\nNoctowl,Flying\n"
    )
}
