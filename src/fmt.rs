use crate::target_triple::TargetTriple;
use ansi_term::Colour;
use ansiterm::Style;
use tracing::info;

pub fn println_error<X: Into<String>>(txt: X) {
    tracing::warn!("{}: {}", Colour::Red.paint("error"), txt.into());
}

pub fn println_warn<X: Into<String>>(txt: X) {
    tracing::warn!("{}: {}", Colour::Yellow.paint("warning"), txt.into());
}

pub fn bold(text: &str) -> String {
    let style = Style::new().bold();
    format!("{}", style.paint(text))
}

pub fn colored_bold(color: ansiterm::Color, text: &str) -> String {
    format!("{}", color.bold().paint(text))
}

pub fn print_header(header: &str) {
    info!("");
    info!("{}", bold(header));
    info!("{}", "-".repeat(header.len()));
}

pub fn format_toolchain_with_target(toolchain: &str) -> String {
    format!(
        "{}-{}",
        toolchain,
        TargetTriple::from_host().unwrap_or_default()
    )
}
