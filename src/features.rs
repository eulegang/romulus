
/// Gives a record of which features romulus was installed with
pub fn features() -> Vec<(bool, String)> {
    vec![
        (cfg!(feature = "color"), "color".to_string()),
        (cfg!(feature = "envvar"), "envvar".to_string()),
        (cfg!(feature = "stdin-tty"), "stdin-tty".to_string()),
    ]
}

