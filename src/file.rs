use anyhow::{Context, Result};
use semver::Version;
use std::{fs, io, os::unix::fs::PermissionsExt, path::Path};

#[cfg(unix)]
pub(crate) fn is_executable(file: &Path) -> bool {
    file.is_file() && file.metadata().unwrap().permissions().mode() & 0o111 != 0
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum BinError {
    #[error("not found")]
    NotFound,

    #[error("Could not parse version ({0})")]
    SemVer(#[from] semver::Error),

    #[error("{0}")]
    Io(#[from] io::Error),
}

pub(crate) fn get_bin_version(exec_path: &Path) -> Result<Version, BinError> {
    if !exec_path.is_file() {
        return Err(BinError::NotFound);
    }
    let output = std::process::Command::new(exec_path)
        .arg("--version")
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    Ok(Version::parse(
        stdout.split_whitespace().last().unwrap_or_default(),
    )?)
}

pub(crate) fn hardlink(original: &Path, link: &Path) -> io::Result<()> {
    let _ = fs::remove_file(link);
    fs::hard_link(original, link)
}

pub fn hard_or_symlink_file(original: &Path, link: &Path) -> Result<()> {
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

pub fn read_file<X: AsRef<Path>>(name: &'static str, path: X) -> Result<String> {
    fs::read_to_string(path).with_context(|| format!("Failed to read {name}"))
}

pub fn write_file<X: AsRef<Path>>(path: X, contents: &str) -> io::Result<()> {
    let mut file = fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(path)?;

    io::Write::write_all(&mut file, contents.as_bytes())?;

    file.sync_data()?;

    Ok(())
}
