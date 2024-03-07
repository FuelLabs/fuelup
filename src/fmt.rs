use std::io::{self, Write};

use crate::target_triple::TargetTriple;
use ansi_term::Colour;
use ansiterm::Style;
use tracing::info;

pub fn println_error<X: Into<String>>(txt: X) {
    tracing::warn!("{}: {}", Colour::Red.paint("error"), txt.into());
}

pub fn println_info<X: Into<String>>(txt: X) {
    tracing::info!("info: {}", txt.into());
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

pub fn ask_user_yes_no_question(question: &str) -> io::Result<bool> {
    loop {
        print!("{question} ");
        std::io::stdout().flush()?;
        let mut ans = String::new();
        std::io::stdin().read_line(&mut ans)?;
        // Pop trailing \n as users press enter to submit their answers.
        ans.pop();
        // Trim the user input as it might have an additional space.
        let ans = ans.trim();
        if let Some(result) = match ans {
            "y" | "Y" => Some(true),
            "n" | "N" => Some(false),
            _ => None,
        } {
            return Ok(result);
        }
    }
}
