# vamsfmt

A standalone Rust formatter for **Verilog-A and core Verilog-AMS 2.4**. It reads
stdin, formats indentation and layout, and writes stdout. It works without Neovim,
Cadence, or a simulator license.

The parser is embedded from the published
[tree-sitter-verilog-ams](https://github.com/nohgo/tree-sitter-verilog-ams) grammar,
pinned to `0b2672e76ff2941fb62081b5d42b0ae84a0f6c46`. Builds do not depend on a
neighboring grammar checkout. See [architecture](docs/ARCHITECTURE.md) for the
source-preservation design and [coverage](docs/COVERAGE.md) for limitations.

## Install from GitHub

Requirements: Rust/Cargo 1.85 or newer, Git, and a C compiler. The first build
fetches the pinned grammar and Cargo dependencies. Node.js and the Tree-sitter CLI
are not needed to build or run the formatter.

Run this from any directory; no local workspace checkout is needed:

```sh
cargo install --git https://github.com/nohgo/vamsfmt.git --branch main --locked
vamsfmt --version
```

Ensure Cargo's executable directory (normally `~/.cargo/bin`) is on `$PATH`.
For a standard Cargo installation, add `export PATH="$HOME/.cargo/bin:$PATH"` to
your shell startup file if needed, then restart your terminal and Neovim. Only the
installed executable is needed at runtime; it does not shell out to the parser
or read the grammar repository. The parser version is pinned inside this binary,
independently of the parser installed for Neovim highlighting.

Try the CLI before configuring the editor:

```sh
printf 'module demo(p,n); inout p,n; electrical p,n; analog begin I(p,n)<+V(p,n)/1k; end endmodule\n' | vamsfmt
```

The result should have indented declarations and an indented analog contribution,
matching the layout in the example below. To format an existing file, write to a
different output path:

```sh
vamsfmt < model.va > model.formatted.va
```

### Updates and removal

Rerun the GitHub install command to install the latest `main`. If switching from
an earlier `cargo install --path` installation, add `--force` once to replace that
installation. For a reproducible revision, replace `--branch main` with
`--rev <commit-sha>`. Keep `--locked` to use the checked-in dependency versions.

Use `cargo uninstall vamsfmt` to remove the binary, and remove its Conform mapping
if you no longer want to use it. Updating Lazy plugins does not update this CLI.

### Build a local checkout

```sh
git clone https://github.com/nohgo/vamsfmt.git
cd vamsfmt
make build
./target/release/vamsfmt < model.va > model.formatted.va
cargo install --path . --locked
```

`make` uses `build/cargo/` as its default Cargo cache, while direct Cargo commands
use your normal Cargo environment.

## Formatting behavior

Defaults: **two spaces** per indentation level and a **soft 100-column** line
width. Core rules cover modules, declarations, analog/digital blocks, control
flow, analog functions, cases, generate blocks, ordinary operator spacing, and
wrapping of ports, parameters, function arguments, and expressions.

```verilog
module resistor(p, n);
  inout p, n;
  electrical p, n;
  parameter real r = 1k from (0:inf);
  analog begin
    I(p, n) <+ V(p, n) / r;
  end
endmodule
```

Use these options when needed:

```sh
vamsfmt --indent-width 4 --line-width 80 < model.va
vamsfmt --pass-through < model.va
```

`--pass-through` still validates syntax, then returns exact original bytes. In
normal mode, constructs without formatting rules retain their original source
verbatim. Existing blank lines, LF/CRLF sequences, and final-newline state are
preserved. Existing multiline layouts stay multiline; the formatter does not
collapse them into one line. Width counts characters, not terminal display cells.

Comments, strings, number spellings, escaped names, attributes, macro calls, and
complete preprocessor lines are preserved. Multiline comment/macro interiors are
never reindented or reflowed. Opaque constructs and indivisible tokens may exceed
the soft width limit. Vertical port/operator column alignment is deferred.

## Safety and exit codes

The formatter parses before editing and rejects every syntax error or missing
token. It rewrites only whitespace between preserved source spans, reparses the
complete result, and checks identical syntax structure, leaf text, and protected
source atoms before emitting anything.

| Exit | Meaning | stdout |
| --- | --- | --- |
| `0` | Success | Complete formatted source |
| `1` | Invalid source or failed preservation check | Empty |
| `2` | Usage, parser initialization, or I/O failure | No formatted source before validation; an output I/O failure can interrupt writing |

Diagnostics go to stderr. `--help` and `--version` print informational output.
There is no in-place mode, include resolution, macro expansion, or semantic
analysis. Unsupported grammar syntax is rejected; an error-free parse is not
proof of simulator correctness. A failed preservation check returns an error
rather than silently applying part of a formatting result.

## Neovim / conform.nvim

1. Install the binary using the GitHub command above.
2. Keep your existing [Verilog-AMS grammar integration](https://github.com/nohgo/tree-sitter-verilog-ams#neovim-install-from-github)
   for highlighting and filetype detection.
3. Add this specification to your `lazy.nvim` plugins. If you already configure
   Conform, merge the formatter and filetype entries into that configuration.

```lua
return {
  'stevearc/conform.nvim',
  opts = {
    formatters = {
      vamsfmt = {
        command = 'vamsfmt',
        args = { '--indent-width', '2', '--line-width', '100' },
        stdin = true,
      },
    },
    formatters_by_ft = { verilogams = { 'vamsfmt' } },
    format_on_save = { timeout_ms = 1000, lsp_format = 'never' },
  },
}
```

The same [configuration fragment](contrib/neovim/conform.lua) is provided as a
file. For setups without Lazy, pass the contents of `opts` to
`require('conform').setup(...)`. Conform runs the installed binary; there is no
`nohgo/vamsfmt` Lua plugin to add to Lazy.

### Verify in Neovim

Run `:Lazy sync`, restart Neovim, and open a `.va` or `.vams` file. Check:

```vim
:set filetype?
:echo executable('vamsfmt')
:ConformInfo
```

Expect `filetype=verilogams`, executable result `1`, and `vamsfmt` available for
the buffer. Use `:echo exepath('vamsfmt')` to locate the binary Neovim sees. A
terminal-only `$PATH` change may require restarting the application that launches
Neovim.

Paste the one-line `module demo(...)` source from the CLI example into a test
buffer, then run:

```vim
:lua require('conform').format({ async = false })
```

It should expand into indented code. Undo and save with `:write` to verify
format-on-save. Change `--indent-width` or `--line-width` in the formatter's
`args` to tune the layout. Remove the `verilogams` formatter mapping to disable
formatting while keeping highlighting and other filetypes' formatters available.

The grammar plugin associates `.va`, `.vams`, and `.vah` with `verilogams`. If
using the formatter without it, register the filetype yourself:

```lua
vim.filetype.add({ extension = { va = 'verilogams', vams = 'verilogams', vah = 'verilogams' } })
```

Conform leaves an invalid buffer unchanged when the formatter exits nonzero.
Normal save behavior still applies: rejecting formatting does not cancel a save.
This follows [Conform's custom formatter interface](https://github.com/stevearc/conform.nvim#customizing-formatters).

## Development and verification

```sh
make test          # golden, grammar regression, preservation, idempotence, CLI tests
make check         # rustfmt and Clippy with warnings treated as errors
make test-neovim   # release build + pinned Conform dependency + headless editor test
```

The editor test fetches Conform commit
`016802de402556da54c36bd7359b441266b01cdd` into `build/test-deps/` and runs Neovim
with `-u NONE -i NONE`, using local cache/log directories. It verifies explicit
formatting, actual format-on-save, and unchanged invalid buffers. It never reads
or edits personal Neovim configuration. Neovim 0.11+ is required for this test.

Tests exercise 74 grammar corpus inputs and six model/header examples, including
all combinations of line widths 20/40/60/100 and indentation widths 2/4. Reviewed
goldens cover layout, and focused tests cover raw fallback, comments/macros,
CRLF, missing final newlines, invalid UTF-8, syntax recovery, and CLI streams.

Locally verified with Rust 1.85.1, Tree-sitter runtime 0.26.13, and Neovim 0.11.5
and 0.12.5.
A release smoke check formatted 500 small modules (about 82 KB) in approximately
0.09 seconds on the development machine; this is not a large-model benchmark.

Add rules only for explicitly supported CST nodes. Add regression/golden cases,
review exact expected layouts, and preserve the raw fallback. See
[fixture provenance](tests/fixtures/README.md) for the test input sources.
