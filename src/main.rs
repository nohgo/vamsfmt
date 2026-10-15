use std::io::{self, Read, Write};
use std::process::ExitCode;
use vamsfmt::Options;

const HELP: &str = "vamsfmt [--indent-width N] [--line-width N] [--pass-through]\n\nRead Verilog-A / Verilog-AMS from stdin; write formatted source to stdout.\n  --indent-width N   Spaces per level (1..16; default 2)\n  --line-width N     Soft line width (20..1000; default 100)\n  --pass-through     Validate and emit exact original source\n  -h, --help         Show this help\n  -V, --version      Show version\n\nExit status: 0 success; 1 rejected source; 2 usage or I/O failure.\n";

fn run() -> Result<(), (u8, String)> {
    let mut options = Options::default();
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => {
                print!("{HELP}");
                return Ok(());
            }
            "-V" | "--version" => {
                println!("vamsfmt {}", env!("CARGO_PKG_VERSION"));
                return Ok(());
            }
            "--pass-through" => options.pass_through = true,
            _ => {
                let (name, value) = match arg.split_once('=') {
                    Some((name, value)) => (name.to_owned(), value.to_owned()),
                    None if arg == "--indent-width" || arg == "--line-width" => {
                        let value = args
                            .next()
                            .ok_or_else(|| (2, format!("{arg} requires a value")))?;
                        (arg.clone(), value)
                    }
                    _ => return Err((2, format!("unknown argument {arg:?}; use --help"))),
                };
                let number = value
                    .parse::<usize>()
                    .map_err(|_| (2, format!("invalid value for {name}: {value:?}")))?;
                match name.as_str() {
                    "--indent-width" => options.indent_width = number,
                    "--line-width" => options.line_width = number,
                    _ => return Err((2, format!("unknown argument {name:?}; use --help"))),
                }
            }
        }
    }
    options
        .validate()
        .map_err(|e| (e.exit_code(), e.to_string()))?;
    let mut input = Vec::new();
    io::stdin()
        .read_to_end(&mut input)
        .map_err(|e| (2, format!("cannot read stdin: {e}")))?;
    let source =
        std::str::from_utf8(&input).map_err(|e| (1, format!("input is not UTF-8: {e}")))?;
    let output = vamsfmt::format(source, options).map_err(|e| (e.exit_code(), e.to_string()))?;
    io::stdout()
        .lock()
        .write_all(output.as_bytes())
        .map_err(|e| (2, format!("cannot write stdout: {e}")))?;
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err((code, message)) => {
            eprintln!("vamsfmt: {message}");
            ExitCode::from(code)
        }
    }
}
