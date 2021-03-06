extern crate regex;
extern crate romulus;

#[macro_use]
mod common;

#[test]
fn regex_selector() {
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
fn selector_capture() {
    check_output!(
        "1 { print('name,type') }\n/pokemon \"(?P<name>.*)\"/,/}/ { /type *= *\"(?P<type>.*)\"/ { print(\"${name},${type}\") } }",
        "pokemon \"Haunter\" {\ntype = \"Ghost\"\n}\npokemon \"Noctowl\" {\ntype = \"Flying\"\n}\n",
        "name,type\nHaunter,Ghost\nNoctowl,Flying\n"
    );
}

#[test]
fn symbolic_anchors() {
    check_output!(
        "^ { print 'first' }; print _ $ { print 'last'  }",
        "middle",
        "first\nmiddle\nlast\n"
    );
}

#[test]
fn symbolic_anchors_empty_input() {
    check_output!(
        "^ print('first') print _ $ print('last') ",
        "",
        "first\nlast\n"
    );
}

#[test]
fn symbolic_anchors_print_removes_implicit_print() {
    check_output!("^ print 'first' $ print 'last'", "middle", "first\nlast\n");
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

#[test]
fn set() {
    check_output!(
        "/set (?P<name>[^ ]+) (?P<val>.*)/ set \"${name} = ${val}\" print _",
        "set x 123\nget x\n",
        "x = 123\nget x\n"
    )
}

#[test]
fn basic_negation() {
    check_output!(
        "!1 print _",
        "hello\nworld\nnice\nto\nmeet\nyou!\n",
        "world\nnice\nto\nmeet\nyou!\n"
    );
}

#[test]
fn negation_range_interaction() {
    check_output!(
        "! 2,/meet/ print _",
        "hello\nworld\nnice\nto\nmeet\nyou!\n",
        "hello\nmeet\nyou!\n"
    );
}

#[test]
fn implicit_print() {
    check_output!(
        "subst /hello/, 'goodbye'",
        "hello world\ncy@\n",
        "goodbye world\ncy@\n"
    );
}

#[test]
fn disabled_implicit_print() {
    check_output!("subst /hello/, 'goodbye'", "hello world\ncy@\n", "", false);
}

#[test]
fn basic_conj() {
    check_output!(
        "/thing/ & /there/ print _",
        "this thing\nthing something there\nthere\n",
        "thing something there\n"
    );
}

#[test]
fn conj_scope() {
    check_output!(
        "/export (?P<name>[a-zA-Z]+)/ & /= (?P<value>.*)/ print \"${name}: ${value}\"",
        "export NAME = VALUE\n",
        "NAME: VALUE\n"
    );
}

#[test]
fn conj_range_interaction() {
    check_output!(
        "/th/ & 2,5 { print _ }",
        "first\nsecond\nthird\nfourth\nfifth\nsexth\nseventh\nninth\ntenth\n",
        "third\nfourth\n"
    );
}

#[test]
fn dist_range_interaction() {
    check_output!(
        "/th/ | 3,5 { print _ }",
        "first\nsecond\nthird\nfourth\nbreaking\npattern\nfor\ntest\n",
        "third\nfourth\n"
    );
}

#[test]
fn selector_disjunction() {
    check_output!(
        "/thing/ | /here/ print _",
        "this thing\nthing something there\nthere\nhello!\n",
        "this thing\nthing something there\nthere\n"
    );
}
