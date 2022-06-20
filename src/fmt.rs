use anyhow::Result;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

pub fn bold<F>(write: F) -> Result<()>
where
    F: FnOnce(&mut StandardStream) -> std::io::Result<()>,
{
    let mut stdout = StandardStream::stdout(ColorChoice::Auto);
    stdout.set_color(ColorSpec::new().set_bold(true))?;
    write(&mut stdout)?;
    stdout.reset()?;
    Ok(())
}

pub fn with_color_maybe_bold<F>(write: F, color: Color, bold: bool) -> Result<()>
where
    F: FnOnce(&mut StandardStream) -> std::io::Result<()>,
{
    let mut stdout = StandardStream::stdout(ColorChoice::Auto);
    stdout.set_color(ColorSpec::new().set_fg(Some(color)).set_bold(bold))?;
    write(&mut stdout)?;
    stdout.reset()?;
    Ok(())
}
