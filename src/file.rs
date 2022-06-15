use anyhow::{Context, Result};
use std::fs;
use std::io;
use std::path::Path;

pub(crate) fn hardlink(original: &Path, link: &Path) -> io::Result<()> {
    let _ = fs::remove_file(link);
    fs::hard_link(original, link)
}

pub(crate) fn hard_or_symlink_file(original: &Path, link: &Path) -> Result<()> {
    if hardlink_file(original, link).is_err() {
        symlink_file(original, link)?;
    }
    Ok(())
}

pub fn hardlink_file(original: &Path, link: &Path) -> Result<()> {
    hardlink(original, link).with_context(|| {
        format!(
            "Could not create link: {}->{}",
            original.display(),
            link.display()
        )
    })
}

#[cfg(unix)]
fn symlink_file(original: &Path, link: &Path) -> Result<()> {
    std::os::unix::fs::symlink(original, link).with_context(|| {
        format!(
            "Could not create link: {}->{}",
            original.display(),
            link.display()
        )
    })
}

#[cfg(not(unix))]
fn symlink_file(_original: &Path, _link: &Path) -> Result<()> {
    bail!("Symbolic link currently only supported on Unix");
}

pub fn is_file<P: AsRef<Path>>(path: P) -> bool {
    fs::metadata(path).ok().as_ref().map(fs::Metadata::is_file) == Some(true)
}

pub fn read_file(name: &'static str, path: &Path) -> Result<String> {
    fs::read_to_string(path).with_context(|| format!("Failed to read {}", name))
}

pub fn write_file(path: &Path, contents: &str) -> io::Result<()> {
    let mut file = fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(path)?;

    io::Write::write_all(&mut file, contents.as_bytes())?;

    file.sync_data()?;

    Ok(())
}
