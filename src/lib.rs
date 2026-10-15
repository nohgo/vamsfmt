//! Source-preserving Verilog-A / Verilog-AMS formatting.
mod layout;
mod syntax;

use std::fmt;

/// Small initial style surface; line width is a soft target.
#[derive(Clone, Copy, Debug)]
pub struct Options {
    pub indent_width: usize,
    pub line_width: usize,
    pub pass_through: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            indent_width: 2,
            line_width: 100,
            pass_through: false,
        }
    }
}

impl Options {
    pub fn validate(self) -> Result<(), Error> {
        if !(1..=16).contains(&self.indent_width) {
            return Err(Error::usage("indent width must be between 1 and 16"));
        }
        if !(20..=1000).contains(&self.line_width) {
            return Err(Error::usage("line width must be between 20 and 1000"));
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct Error {
    message: String,
    exit_code: u8,
}

impl Error {
    pub(crate) fn rejected(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            exit_code: 1,
        }
    }
    pub(crate) fn usage(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            exit_code: 2,
        }
    }
    pub fn exit_code(&self) -> u8 {
        self.exit_code
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.message.fmt(f)
    }
}
impl std::error::Error for Error {}

/// Format a complete source string. No output is returned before validation.
pub fn format(source: &str, options: Options) -> Result<String, Error> {
    options.validate()?;
    let mut parser = syntax::parser()?;
    let original = syntax::parse(&mut parser, source)?;
    if options.pass_through || source.trim().is_empty() {
        return Ok(source.to_owned());
    }
    let output = layout::format(source, original.root_node(), options)?;
    let candidate = syntax::parse(&mut parser, &output)
        .map_err(|e| Error::rejected(format!("unsafe formatting rejected: {e}")))?;
    if syntax::signature(original.root_node(), source)
        != syntax::signature(candidate.root_node(), &output)
    {
        return Err(Error::rejected(
            "unsafe formatting rejected: syntax structure or token text changed",
        ));
    }
    if layout::atoms(source, original.root_node()) != layout::atoms(&output, candidate.root_node())
    {
        return Err(Error::rejected(
            "unsafe formatting rejected: protected source text changed",
        ));
    }
    Ok(output)
}
