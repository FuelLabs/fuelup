use anyhow::{Context, Result};
use std::fs;
use std::io;
use std::path::Path;

pub(crate) fn hardlink(src: &Path, dest: &Path) -> io::Result<()> {
    let _ = fs::remove_file(dest);
    fs::hard_link(src, dest)
}

pub(crate) fn hard_or_symlink_file(src: &Path, dest: &Path) -> Result<()> {
    if hardlink_file(src, dest).is_err() {
        symlink_file(src, dest)?;
    }
    Ok(())
}

pub fn hardlink_file(src: &Path, dest: &Path) -> Result<()> {
    hardlink(src, dest).with_context(|| "Could not create link".to_string())
}

#[cfg(unix)]
fn symlink_file(src: &Path, dest: &Path) -> Result<()> {
    std::os::unix::fs::symlink(src, dest).with_context(|| "Could not create link".to_string())
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
