use anyhow::{Context, Result};
use semver::Version;
use std::{
    fs, io,
    os::unix::fs::{MetadataExt, PermissionsExt},
    path::Path,
};

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
    if links_same_file(original, link)? {
        return Ok(());
    }

    let _ = fs::remove_file(link);
    match fs::hard_link(original, link) {
        Ok(()) => Ok(()),
        Err(error)
            if error.kind() == io::ErrorKind::AlreadyExists
                && links_same_file(original, link)? =>
        {
            Ok(())
        }
        Err(error) => Err(error),
    }
}

fn links_same_file(original: &Path, link: &Path) -> io::Result<bool> {
    let original = fs::metadata(original)?;
    let link = match fs::metadata(link) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(error),
    };

    Ok(original.dev() == link.dev() && original.ino() == link.ino())
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Barrier};

    #[test]
    fn existing_hardlink_is_safe_under_parallel_reuse() {
        const THREADS: usize = 32;
        const ITERATIONS: usize = 100;

        let temp_dir = tempfile::tempdir().unwrap();
        let original = Arc::new(temp_dir.path().join("original"));
        let link = Arc::new(temp_dir.path().join("link"));
        fs::write(original.as_ref(), "fuelup").unwrap();
        fs::hard_link(original.as_ref(), link.as_ref()).unwrap();

        let barrier = Arc::new(Barrier::new(THREADS));
        let workers: Vec<_> = (0..THREADS)
            .map(|_| {
                let original = Arc::clone(&original);
                let link = Arc::clone(&link);
                let barrier = Arc::clone(&barrier);
                std::thread::spawn(move || {
                    barrier.wait();
                    for _ in 0..ITERATIONS {
                        hardlink(&original, &link).unwrap();
                    }
                })
            })
            .collect();

        for worker in workers {
            worker.join().unwrap();
        }

        assert!(links_same_file(&original, &link).unwrap());
        assert_eq!(fs::read_to_string(link.as_ref()).unwrap(), "fuelup");
    }
}
