use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

// In the below functions, we ignore the `Result`s of `set_color` and `reset` to allow `write`
// to work even when those functions fail to set/reset colors, since `StandardStream::stdout` is
// a wrapper over `std::io::stdout`.

pub fn bold<F>(write: F)
where
    F: FnOnce(&mut StandardStream) -> std::io::Result<()>,
{
    let mut stdout = StandardStream::stdout(ColorChoice::Auto);
    let _ = stdout.set_color(ColorSpec::new().set_bold(true));
    write(&mut stdout).expect("Unexpected error writing to stdout");
    let _ = stdout.reset();
}

pub fn colored_bold<F>(color: Color, write: F)
where
    F: FnOnce(&mut StandardStream) -> std::io::Result<()>,
{
    let mut stdout = StandardStream::stdout(ColorChoice::Auto);
    let _ = stdout.set_color(ColorSpec::new().set_fg(Some(color)).set_bold(true));
    write(&mut stdout).expect("Unexpected error writing to stdout");
    let _ = stdout.reset();
}
