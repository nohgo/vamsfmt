# vamsfmt architecture

## Scope and technology

Build a standalone Rust CLI, independent of Neovim and Cadence. Embed the C parser
through the grammar's Rust binding. Pin `tree-sitter-verilog-ams` to published Git
revision `0b2672e76ff2941fb62081b5d42b0ae84a0f6c46` and use Tree-sitter runtime
0.26.13. Builds must not depend on the neighboring workspace checkout.

The first formatting rules target indentation, statement/block layout, ordinary
operator spacing, and conservative wrapping of ports, parameters, call arguments,
and long expressions. Default to two spaces and a soft 100-column limit. Expose
only indentation width and line width initially; defer vertical column alignment,
comment reflow, configuration discovery, and in-place editing.

## Safety pipeline

1. Read all stdin as UTF-8. Parse before planning any edits. Reject every `ERROR`
   or missing token, including recoverable syntax errors.
2. Traverse an explicit allowlist of supported CST productions. Record source
   atoms, byte ranges, layout boundaries, and indentation levels. An unhandled
   node becomes one opaque atom containing its exact original source; do not
   recursively format inside it.
3. Preserve strings, escaped identifiers, numeric literals, comments, attributes,
   macro calls, and complete preprocessor directive lines as indivisible source
   slices. Never expand macros or resolve includes. Preserve line-comment
   attachment and required whitespace after escaped identifiers.
4. Rewrite only whitespace gaps between atoms. Add block/statement boundaries,
   indent from CST nesting, expand long or existing multiline lists, then wrap at
   known comma/operator boundaries. Width is soft: opaque text, comments, and
   indivisible tokens may exceed it. Retain existing blank lines and final-newline
   state; preserve existing newline sequences and use the first newline style for
   inserted breaks.
5. Reparse the complete candidate. Require an error-free tree, identical CST
   structure and leaf text, and identical atom kinds/text (including opaque
   subtrees). Reject any candidate that fails these invariants. These checks
   guard source preservation; they are not a simulator equivalence proof.
6. Write stdout only after every check passes. Library formatting returns a
   complete string or an error. The CLI never modifies input files.

Keep a `--pass-through` mode that validates syntax and returns exact original
bytes. Parsed but unformatted constructs use the same raw-source fallback in
normal mode. Syntax the grammar cannot parse must be rejected, not passed through
as if validated.

## Code boundaries

- `src/lib.rs`: public options and formatting pipeline.
- `src/syntax.rs`: parser setup, error locations, and structural verification.
- `src/layout.rs`: allowlisted CST traversal, opaque atoms, whitespace layout,
  indentation, and wrapping.
- `src/main.rs`: small stdin/stdout adapter and argument handling.
- `tests/`: golden layouts, grammar regression fixtures, preservation/idempotence,
  real CLI status/stream checks, and headless Conform integration.
- `contrib/neovim/`: explicit Conform configuration examples; no automatic editor
  or personal configuration changes.

## CLI and editor contract

`vamsfmt [--indent-width N] [--line-width N] [--pass-through]` reads stdin and emits
formatted source to stdout. Exit 0 means success, 1 means invalid/rejected source
or a failed safety check, and 2 means usage or I/O failure. Diagnostics go to
stderr; formatting failures produce no stdout. `--help` and `--version` are
ordinary informational exceptions.

Conform uses a custom formatter with `stdin = true` and a
`formatters_by_ft.verilogams` entry. Verify both successful replacement and an
unchanged invalid buffer in an isolated Neovim session with the actual plugin.

## Delivery gates

First establish syntax rejection and identity output. Then implement layout rules
against reviewed input/output fixtures. Exercise all 74 existing grammar corpus
cases and model examples for successful formatting, reparsing, and idempotence.
Include macros/continuations, comments, unhandled syntax, escaped names, LF/CRLF,
Unicode, whitespace-only input, and missing final newlines. Verify the CLI, release
binary, and Conform before reporting completion. Leave grammar and Cadence work
untouched unless a concrete compatibility defect requires a separate change.
