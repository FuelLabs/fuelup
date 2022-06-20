use anyhow::Result;
use std::io::Write;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

pub fn print_boldln(txt: &str) -> Result<()> {
    let mut stdout = StandardStream::stdout(ColorChoice::Auto);
    stdout.set_color(ColorSpec::new().set_bold(true))?;

    writeln!(&mut stdout, "{}", txt)?;
    stdout.reset()?;
    Ok(())
}

pub fn print_bold(txt: &str) -> Result<()> {
    let mut stdout = StandardStream::stdout(ColorChoice::Auto);
    stdout.set_color(ColorSpec::new().set_bold(true))?;

    write!(&mut stdout, "{}", txt)?;
    stdout.reset()?;
    Ok(())
}

pub fn print_with_color(txt: &str, color: Color) -> Result<()> {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    stdout.set_color(ColorSpec::new().set_fg(Some(color)).set_bold(true))?;

    write!(&mut stdout, "{}", txt)?;
    stdout.reset()?;
    Ok(())
}
