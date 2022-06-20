use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

pub fn bold<F>(write: F)
where
    F: FnOnce(&mut StandardStream) -> std::io::Result<()>,
{
    let mut stdout = StandardStream::stdout(ColorChoice::Auto);
    stdout.set_color(ColorSpec::new().set_bold(true)).ok();
    write(&mut stdout).expect("Unexpected error writing to stdout");
    stdout.reset().ok();
}

pub fn with_color_maybe_bold<F>(write: F, color: Color, bold: bool)
where
    F: FnOnce(&mut StandardStream) -> std::io::Result<()>,
{
    let mut stdout = StandardStream::stdout(ColorChoice::Auto);
    stdout
        .set_color(ColorSpec::new().set_fg(Some(color)).set_bold(bold))
        .ok();
    write(&mut stdout).expect("Unexpected error writing to stdout");
    stdout.reset().ok();
}
