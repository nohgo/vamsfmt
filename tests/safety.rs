use std::io::Write;
use std::process::{Command, Stdio};
use vamsfmt::{format, Options};

fn fixtures() -> Vec<(String, String)> {
    let mut files: Vec<_> = std::fs::read_dir(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/grammar"
    ))
    .unwrap()
    .map(|p| p.unwrap().path())
    .filter(|p| p.extension().is_some())
    .collect();
    files.sort();
    files
        .into_iter()
        .map(|p| {
            (
                p.file_name().unwrap().to_string_lossy().to_string(),
                std::fs::read_to_string(p).unwrap(),
            )
        })
        .collect()
}

#[test]
fn validated_pass_through_is_exact_for_every_grammar_fixture() {
    let options = Options {
        pass_through: true,
        ..Options::default()
    };
    for (name, source) in fixtures() {
        assert_eq!(
            format(&source, options).unwrap_or_else(|e| panic!("{name}: {e}")),
            source,
            "{name}"
        );
    }
}

#[test]
fn grammar_fixtures_format_and_are_idempotent() {
    for (name, source) in fixtures() {
        let output = format(&source, Options::default()).unwrap_or_else(|e| panic!("{name}: {e}"));
        assert_eq!(
            format(&output, Options::default()).unwrap_or_else(|e| panic!("{name}: {e}")),
            output,
            "{name}"
        );
    }
}

#[test]
fn grammar_fixtures_are_stable_across_indent_and_width_settings() {
    for (name, source) in fixtures() {
        for width in [20, 40, 60, 100] {
            for indent in [2, 4] {
                let options = Options {
                    line_width: width,
                    indent_width: indent,
                    ..Options::default()
                };
                let output = format(&source, options)
                    .unwrap_or_else(|e| panic!("{name}, width {width}, indent {indent}: {e}"));
                assert_eq!(
                    format(&output, options).unwrap(),
                    output,
                    "{name}, width {width}, indent {indent}"
                );
            }
        }
    }
}

fn run(input: &[u8], args: &[&str]) -> std::process::Output {
    let mut process = Command::new(env!("CARGO_BIN_EXE_vamsfmt"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    process.stdin.take().unwrap().write_all(input).unwrap();
    process.wait_with_output().unwrap()
}

#[test]
fn syntax_failures_emit_no_stdout_in_both_modes() {
    for input in [
        "module m; analog begin x=1 end endmodule", // missing recovery token
        "module m; analog begin V(p)<+; end endmodule",
        "module m; analog begin V(p)<+1; endmodule",
        "module m; electrical p analog V(p)<+1; endmodule",
        "module m; parameter real r; endmodule",
        "module first; wire a; module second; wire b; endmodule",
        "module m; begin n; endmodule",
    ] {
        for args in [vec![], vec!["--pass-through"]] {
            let result = run(input.as_bytes(), &args);
            assert_eq!(result.status.code(), Some(1));
            assert!(result.stdout.is_empty());
            assert!(!result.stderr.is_empty());
        }
    }
}

#[test]
fn cli_streams_usage_and_encoding_contract() {
    let source = b"module m; endmodule\n";
    let result = run(source, &["--pass-through"]);
    assert!(result.status.success());
    assert_eq!(result.stdout, source);
    assert!(result.stderr.is_empty());
    for args in [
        vec!["--unknown"],
        vec!["--indent-width", "0"],
        vec!["--line-width=1"],
        vec!["file.va"],
    ] {
        let result = run(b"", &args);
        assert_eq!(result.status.code(), Some(2));
        assert!(result.stdout.is_empty());
    }
    let result = run(&[0xff, 0xfe], &[]);
    assert_eq!(result.status.code(), Some(1));
    assert!(result.stdout.is_empty());
}

#[test]
fn whitespace_only_inputs_remain_exact() {
    for source in ["", " \t\n\r\n  ", "\r\n"] {
        assert_eq!(format(source, Options::default()).unwrap(), source);
    }
}
