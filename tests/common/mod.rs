#[macro_export]
macro_rules! check_output {
    ($prog: expr, $input: expr, $expected: expr) => {{
        let interpreter = romulus::Interpreter::builder()
            .expression($prog.to_string())
            .sep(regex::Regex::new(" +").unwrap())
            .print(true)
            .build()
            .unwrap();

        let mut out = Vec::new();
        let mut sin = $input.as_bytes();

        interpreter.process(&mut sin, &mut out);

        let actual_expected = if cfg!(target_os = "windows") {
            $expected.replace("\n", "\r\n")
        } else {
            $expected.to_string()
        };

        assert_eq!(String::from_utf8(out).unwrap(), actual_expected);
    }};

    ($prog: expr, $input: expr, $expected: expr, $implicit: expr) => {{
        let interpreter = romulus::Interpreter::builder()
            .expression($prog.to_string())
            .sep(regex::Regex::new(" +").unwrap())
            .print($implicit)
            .build()
            .unwrap();

        let mut out = Vec::new();
        let mut sin = $input.as_bytes();

        interpreter.process(&mut sin, &mut out);

        let actual_expected = if cfg!(target_os = "windows") {
            $expected.replace("\n", "\r\n")
        } else {
            $expected.to_string()
        };

        assert_eq!(String::from_utf8(out).unwrap(), actual_expected);
    }};
}
