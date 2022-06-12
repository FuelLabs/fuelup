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
    hardlink(src, dest).with_context(|| format!("Could not create link"))
}

#[cfg(unix)]
fn symlink_file(src: &Path, dest: &Path) -> Result<()> {
    std::os::unix::fs::symlink(src, dest).with_context(|| format!("Could not create link"))
}
