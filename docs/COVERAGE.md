# Initial formatting coverage

The formatter uses the same Verilog-A / Verilog-AMS 2.4 grammar as the editor.
It is deliberately incremental: recognized syntax without layout support remains
an exact raw subtree; syntax the pinned parser cannot understand is rejected.

| Area | Initial rules |
| --- | --- |
| Modules and ports | Body indentation, separate items, ANSI/non-ANSI headers, long port/parameter lists |
| Analog behavior | `analog` and `analog initial`, begin/end, contributions, events, assignments |
| Control flow | Nested if/else, loops, cases, unbraced bodies, digital procedural blocks |
| Functions | Traditional analog function declarations, arguments/locals, nested bodies; basic digital functions/tasks |
| Declarations | Parameters/ranges/exclusions, nets, branches, variables, basic arrays and initializers |
| Mixed-signal | Core connect modules/rules and generate regions/loops |
| Expressions | Ordinary spacing, comma-separated lists, known operator boundaries, conservative hanging indentation |
| Preservation | Comments, macro uses, full directive/continuation lines, attributes, number/string spellings, opaque nodes |

## Intentional limits

- Width is soft. Never split strings, identifiers, comments, directives, or opaque
  constructs to force a line to fit. Preserve existing line breaks and blank-line
  counts; do not reflow comments or normalize the entire file's newline style.
- Preserve complete macro calls and preprocessor lines, including their original
  indentation. Code inside syntactically complete conditional branches can still
  be indented. Token-splicing macros and incomplete constructs may be rejected by
  the grammar even when a simulator's preprocessor could accept them.
- Rare UDP/specify/gate forms and unhandled nested productions keep their original
  internal layout. Ancestors may place the opaque construct on an indented line,
  but its source span is copied verbatim. An internal CST nesting depth of 128
  also switches to raw fallback rather than recursing without bound.
- No vertical column alignment, automatic begin/end insertion, syntax repair,
  comment wrapping, import ordering, semantic checks, or simulator integration.
- Wrapping is a conservative greedy layout, not a global search for the prettiest
  arrangement. Narrow widths can produce many short lines. Common default-width
  layouts are the main target.
- Reparse and preservation checks are required on every normal formatting call.
  They are guards against corruption, not formal proof of behavioral equivalence.

## Extending the formatter

Add a dedicated layout rule or opt a well-understood wrapper into the transparent
allowlist in `src/layout.rs`. Keep unsupported descendants opaque. Update golden
fixtures for indentation/wrapping, add a preservation case when comments or macros
are involved, and run the complete tests plus Conform integration. Grammar/runtime
updates require a deliberate pin change and review of CST differences.
