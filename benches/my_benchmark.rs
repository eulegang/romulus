#[macro_use]
extern crate criterion;

extern crate romulus;

use criterion::Criterion;
use romulus::Interpreter;

fn bench_interpreter_run(bench: &mut Criterion) {
    let prog = "
    ^ print '# bash transformation'
    /export (?P<name>[a-zA-Z0-9]+) = (?P<value>.*)/
      print \"set -x ${name} ${value}\"
    $ print '# end translation'
    ";

    let bash_export =
        "export PATH = '/bin:/sbin:/usr/bin'\nexport LDPATH = ''\n export PYTHONPATH = ''\n";

    let interpreter = Interpreter::builder()
        .expression(prog.to_string())
        .build()
        .unwrap();

    bench.bench_function("bash_translation", |b| {
        b.iter(|| {
            let mut sin = bash_export.as_bytes();
            let mut out = Vec::new();
            interpreter.process(&mut sin, &mut out);
        })
    });
}

fn bench_interpreter_parsing(bench: &mut Criterion) {
    let prog = "
    ^ print '# bash transformation'
    /export (?P<name>[a-zA-Z0-9]+) = (?P<value>.*)/
      print \"set -x ${name} ${value}\"
    $ print '# end translation'
    ";

    bench.bench_function("parsing", |b| {
        b.iter(|| {
            let _ = Interpreter::builder().expression(prog.to_string()).build();
        })
    });
}

criterion_group!(benches, bench_interpreter_run, bench_interpreter_parsing);
criterion_main!(benches);
