use ansiterm::Style;
use tracing::info;

use crate::target_triple::TargetTriple;

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
