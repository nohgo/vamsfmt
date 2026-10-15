use vamsfmt::{format, Options};

#[test]
fn reviewed_layouts() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden");
    for name in [
        "blocks",
        "natures",
        "controls",
        "functions",
        "ports",
        "comments",
        "cases",
    ] {
        let input = std::fs::read_to_string(dir.join(format!("{name}.input.va"))).unwrap();
        let expected = std::fs::read_to_string(dir.join(format!("{name}.expected.va"))).unwrap();
        let output = format(&input, Options::default()).unwrap_or_else(|e| panic!("{name}: {e}"));
        assert_eq!(output, expected, "{name}");
        assert_eq!(
            format(&output, Options::default()).unwrap(),
            output,
            "{name}: idempotence"
        );
    }
}

#[test]
fn custom_indent_and_crlf_without_final_newline() {
    let input = "module m;\r\nanalog begin\r\nI(p)<+1;\r\nend\r\nendmodule";
    let expected = "module m;\r\n    analog begin\r\n        I(p) <+ 1;\r\n    end\r\nendmodule";
    let output = format(
        input,
        Options {
            indent_width: 4,
            ..Options::default()
        },
    )
    .unwrap();
    assert_eq!(output, expected);
    assert!(!output.ends_with('\n'));
}

#[test]
fn unsupported_constructs_and_macro_calls_pass_through_verbatim() {
    let raw = "specify\n   (a => b) = 1;\n endspecify";
    let input = format!("module m;\n{raw}\nanalog begin V(p)<+`SCALE( 1 +  2 );end\nendmodule\n");
    let output = format(&input, Options::default()).unwrap();
    assert!(output.contains(raw));
    assert!(output.contains("`SCALE( 1 +  2 )"));
    assert!(output.contains("    V(p) <+"));
    assert_eq!(format(&output, Options::default()).unwrap(), output);
}

#[test]
fn wrapping_lists_and_expressions_is_stable_at_multiple_widths() {
    let input="module m;electrical p;analog begin V(p)<+transition(V(positive_input,negative_input)*gain+offset,0,1n,1n);end endmodule\n";
    for width in [40, 60, 80, 100] {
        let options = Options {
            line_width: width,
            ..Options::default()
        };
        let output = format(input, options).unwrap();
        assert_eq!(
            format(&output, options).unwrap(),
            output,
            "{width}:\n{output}"
        );
        assert!(
            output.lines().all(|line| line.chars().count() <= width),
            "{width}:\n{output}"
        );
    }
}

#[test]
fn escaped_names_and_conditional_directive_lines_keep_their_bytes() {
    let input = "module m(\\p+ ,n);\ninout \\p+ ,n;electrical \\p+ ,n;\nanalog begin\n   `ifdef ACTIVE\nI(\\p+ ,n)<+1;\n   `else\nI(\\p+ ,n)<+0;\n   `endif\nend\nendmodule";
    let output = format(input, Options::default()).unwrap();
    for raw in ["   `ifdef ACTIVE\n", "   `else\n", "   `endif\n", "\\p+ "] {
        assert!(output.contains(raw));
    }
    assert!(output.contains("    I(\\p+ , n) <+ 1;"));
    assert_eq!(format(&output, Options::default()).unwrap(), output);
}

#[test]
fn unsupported_top_level_primitive_is_exact() {
    let input = "primitive passthrough(q,d);\n output q; input d;\n table\n 0 : 0;\n 1 : 1;\n endtable\nendprimitive\n";
    assert_eq!(format(input, Options::default()).unwrap(), input);
}

#[test]
fn deeply_nested_syntax_switches_to_raw_fallback_safely() {
    let expression = format!("{}1{}", "(".repeat(200), ")".repeat(200));
    let input = format!("module m; analog V(p)<+{expression}; endmodule\n");
    let output = format(&input, Options::default()).unwrap();
    assert!(output.contains(&expression));
    assert_eq!(format(&output, Options::default()).unwrap(), output);
}
