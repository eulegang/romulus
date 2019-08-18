extern crate regex;
extern crate romulus;

use regex::Regex;
use romulus::Interpreter;

macro_rules! check_output {
    ($prog: expr, $input: expr, $expected: expr) => {
        {
            let interpreter = Interpreter::new($prog, Regex::new(" +").unwrap()).unwrap();

            let mut out = Vec::new();
            let mut sin = $input.as_bytes();

            interpreter.process(&mut sin, &mut out);

            let actual_expected = if cfg!(target_os = "windows") {
                $expected.replace("\n", "\r\n")
            } else {
                $expected.to_string()
            };

            assert_eq!(String::from_utf8(out).unwrap(), actual_expected);
        }
    }
}

#[test]
fn basic() {
    check_output!(
        "/needle/ { print('found it') }",
        "hay\nhay\nhey\nneedle\nhay",
        "found it\n"
    );
}

#[test]
fn range() {
    check_output!(
        "/third/,/fifth/ { print _ }",
        "first\nsecond\nthird\nfourth\nfifth\nsexth\nseventh\nninth\ntenth\n",
        "third\nfourth\n"
    );
}

#[test]
fn capture() {
    check_output!(
        "1 { print('name,type') }\n/pokemon \"(?P<name>.*)\"/,/}/ { /type *= *\"(?P<type>.*)\"/ { print(\"${name},${type}\") } }",
        "pokemon \"Haunter\" {\ntype = \"Ghost\"\n}\npokemon \"Noctowl\" {\ntype = \"Flying\"\n}\n",
        "name,type\nHaunter,Ghost\nNoctowl,Flying\n"
    );
}

#[test]
fn symbolic_anchors() {
    check_output!(
        "^ { print 'first' }; // { print _ }; $ { print 'last'  }",
        "middle",
        "first\nmiddle\nlast\n"
    );

    check_output!(
        "^ { print 'first' }; print _; $ { print 'last'  }",
        "middle",
        "first\nmiddle\nlast\n"
    );

    check_output!(
        "^ { print('first') }; // { print _ }; $ { print('last') }",
        "",
        "first\nlast\n"
    );
}

#[test]
fn capture_groups() {
    check_output!(
        "['<none>', _, id] { print id }",
        "<none>                                     <none>              e8de0ade2a84        4 months ago        939MB\n<none>                                     <none>              a6595f96c20b        4 months ago        939MB\n<none>                                     <none>              5e0c040d4ed2        4 months ago        939MB\n<none>                                     <none>              8bd309d63e40        4 months ago        939MB\n<none>                                     <none>              ec6f20e8cd4e        4 months ago        939MB\n<none>                                     <none>              2962fce1c8c3        4 months ago        939MB\n<none>                                     <none>              a20624c0aa07        4 months ago        939MB\n<none>                                     <none>              c8214373eb1b        4 months ago        939MB\nubuntu                                     19.05               a3cb70e64afb        36 months ago       222MB",
        "e8de0ade2a84\na6595f96c20b\n5e0c040d4ed2\n8bd309d63e40\nec6f20e8cd4e\n2962fce1c8c3\na20624c0aa07\nc8214373eb1b\n"
    );
}

#[test]
fn quit() {
    check_output!(
        "/quit/ { quit } /^print: (?P<thing>.*)$/ { print thing }",
        "print: ping\nprint: ping\nprint: ping\nprint: quit\nprint: blarg",
        "ping\nping\nping\n"
    );
}

#[test]
fn subst() {
    check_output!(
        "/blarg/ { subst(/blarg (?P<name>[a-zA-Z0-9]+)/, \"ping ${name}\") print _ } ",
        "ping x; blarg yz; blarg xyz\n",
        "ping x; ping yz; blarg xyz\n"
    );
}

#[test]
fn gsubst() {
    check_output!(
        "/blarg/ { gsubst(/blarg (?P<name>[a-zA-Z0-9]+)/, \"ping ${name}\") print _ } ",
        "ping x; blarg yz; blarg xyz\n",
        "ping x; ping yz; ping xyz\n"
    );
}

#[test]
fn single() {
    check_output!(
        "^ print 'first'\n// print _\n$ print 'last'\n",
        "middle",
        "first\nmiddle\nlast\n"
    );
}

#[test]
fn exec() {
    check_output!(
        "^ exec 'echo first'\n// exec \"echo ${_}\"\n$ exec 'echo last'\n",
        "middle",
        "first\nmiddle\nlast\n"
    );
}

#[test]
fn append() {
    check_output!(
        "/blarg/ append(' blarg blarg') print _",
        "ping\nblarg\n",
        "ping\nblarg blarg blarg\n"
    )
}
