extern crate regex;
extern crate romulus;

use regex::Regex;
use romulus::Interpreter;

macro_rules! run_interpreter {
    ($prog: expr, $input: expr) => {{
        let interpreter = Interpreter::new($prog, Regex::new(" +").unwrap()).unwrap();
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
            "/third/,/fifth/ { print _ }",
            "first\nsecond\nthird\nfourth\nfifth\nsexth\nseventh\nninth\ntenth\n"
        ),
        "third\nfourth\n"
    );
}

#[test]
fn capture() {
    assert_eq!(
        run_interpreter!(
            "1 { print('name,type') }\n/pokemon \"(?P<name>.*)\"/,/}/ { /type *= *\"(?P<type>.*)\"/ { print(\"${name},${type}\") } }",
            "pokemon \"Haunter\" {\ntype = \"Ghost\"\n}\npokemon \"Noctowl\" {\ntype = \"Flying\"\n}\n"
        ),
        "name,type\nHaunter,Ghost\nNoctowl,Flying\n"
    )
}

#[test]
fn symbolic_anchors() {
    assert_eq!(
        run_interpreter!(
            "^ { print 'first' }; // { print _ }; $ { print 'last'  }",
            "middle"
        ),
        "first\nmiddle\nlast\n".to_string()
    );

    assert_eq!(
        run_interpreter!(
            "^ { print 'first' }; print _; $ { print 'last'  }",
            "middle"
        ),
        "first\nmiddle\nlast\n".to_string()
    );

    assert_eq!(
        run_interpreter!(
            "^ { print('first') }; // { print _ }; $ { print('last') }",
            ""
        ),
        "first\nlast\n".to_string()
    );
}

#[test]
fn capture_groups() {
    assert_eq!(
        run_interpreter!(
            "['<none>', _, id] { print id }",
            "<none>                                     <none>              e8de0ade2a84        4 months ago        939MB\n<none>                                     <none>              a6595f96c20b        4 months ago        939MB\n<none>                                     <none>              5e0c040d4ed2        4 months ago        939MB\n<none>                                     <none>              8bd309d63e40        4 months ago        939MB\n<none>                                     <none>              ec6f20e8cd4e        4 months ago        939MB\n<none>                                     <none>              2962fce1c8c3        4 months ago        939MB\n<none>                                     <none>              a20624c0aa07        4 months ago        939MB\n<none>                                     <none>              c8214373eb1b        4 months ago        939MB\nubuntu                                     19.05               a3cb70e64afb        36 months ago       222MB"
        ),
        "e8de0ade2a84\na6595f96c20b\n5e0c040d4ed2\n8bd309d63e40\nec6f20e8cd4e\n2962fce1c8c3\na20624c0aa07\nc8214373eb1b\n"

    );
}

#[test]
fn quit() {
    assert_eq!(
        run_interpreter!(
            "/quit/ { quit } /^print: (?P<thing>.*)$/ { print thing }",
            "print: ping\nprint: ping\nprint: ping\nprint: quit\nprint: blarg"
        ),
        "ping\nping\nping\n"
    )
}

#[test]
fn subst() {
    assert_eq!(
        run_interpreter!(
            "/blarg/ { subst(/blarg (?P<name>[a-zA-Z0-9]+)/, \"ping ${name}\") print _ } ",
            "ping x; blarg yz; blarg xyz\n"
        ),
        "ping x; ping yz; blarg xyz\n"
    );
}

#[test]
fn gsubst() {
    assert_eq!(
        run_interpreter!(
            "/blarg/ { gsubst(/blarg (?P<name>[a-zA-Z0-9]+)/, \"ping ${name}\") print _ } ",
            "ping x; blarg yz; blarg xyz\n"
        ),
        "ping x; ping yz; ping xyz\n"
    );
}

#[test]
fn single() {
    assert_eq!(
        run_interpreter!(
            "^ print 'first'\n// print _\n$ print 'last'\n",
            "middle"
        ),
        "first\nmiddle\nlast\n".to_string()
    );
}

#[test]
fn exec() {
    // echo should exist on every platform
    assert_eq!(
        run_interpreter!(
            "^ exec 'echo first'\n// exec \"echo ${_}\"\n$ exec 'echo last'\n",
            "middle"
        ),
        "first\nmiddle\nlast\n".to_string()
    );
}
