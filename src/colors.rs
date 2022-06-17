use std::io::Write;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

pub fn print_with_color(txt: &str, color: Color) {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    stdout
        .set_color(ColorSpec::new().set_fg(Some(color)))
        .expect("internal printing error");

    write!(&mut stdout, "{}", txt).expect("internal printing error");
    stdout.reset().expect("internal printing error");
}
