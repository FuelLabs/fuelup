use anyhow::{bail, Context, Result};
use component::{self, Components};
use std::{
    fs::{self, remove_dir_all},
    path::Path,
};
use tempfile;
use tracing::{error, info};

use crate::{
    download::{download_file_and_unpack, unpack_bins, DownloadCfg},
    file::{get_bin_version, hard_or_symlink_file, read_file, write_file},
    fmt::{ask_user_yes_no_question, println_info},
    path::{canonical_fuelup_dir, fuelup_bin, fuelup_bin_dir, fuelup_dir, FUELUP_DIR},
    shell::Shell,
    target_triple::TargetTriple,
};

pub fn attempt_install_self(download_cfg: DownloadCfg, dst: &Path) -> Result<()> {
    download_file_and_unpack(&download_cfg, dst)?;
    unpack_bins(dst, dst)?;

    Ok(())
}

#[inline]
fn remove_path_from_content(file_content: &str) -> (bool, String) {
    let mut is_modified = false;
    let whole_definition = format!("PATH={}", FUELUP_DIR);
    let suffix = format!("{}:", FUELUP_DIR);
    let prefix = format!("{}:", FUELUP_DIR);
    let lines = file_content
        .trim_end_matches('\n')
        .trim_end_matches('\r')
        .lines()
        .collect::<Vec<_>>();
    let total_lines = lines.len();
    let new_lines = lines
        .into_iter()
        .filter(|line| !line.trim_end().ends_with(&whole_definition))
        .map(|line| {
            if line.contains("PATH") && line.contains(FUELUP_DIR) {
                is_modified = true;
                line.trim()
                    .replace(&suffix, "")
                    .replace(&prefix, "")
                    .replace(FUELUP_DIR, "")
            } else {
                line.to_owned()
            }
        })
        .collect::<Vec<_>>();
    (
        is_modified || total_lines != new_lines.len(),
        new_lines.join("\n"),
    )
}

/// Removes the fuelup directory from $PATH
fn remove_fuelup_from_path() -> Result<()> {
    for shell in Shell::all() {
        for rc in shell.rc_files().into_iter().filter(|c| c.is_file()) {
            let (was_modified, new_content) = remove_path_from_content(&read_file("rcfile", &rc)?);
            if was_modified {
                println_info(format!(
                    "{} has been updated to remove fuelup from $PATH",
                    rc.display()
                ));
                write_file(rc, &new_content)?;
            }
        }
    }
    Ok(())
}

pub fn self_uninstall(force: bool) -> Result<()> {
    println!(
        r#"Thanks for hacking in Sway!
This will uninstall all Sway toolchains and data, and remove, {}/bin from your PATH environment variable."#,
        canonical_fuelup_dir()?,
    );
    if force || ask_user_yes_no_question("Continue? (y/N)").context("Console I/O")? {
        let remove = [
            ("removing fuelup binaries", fuelup_bin_dir()),
            ("removing fuelup home", fuelup_dir()),
        ];
        remove_fuelup_from_path()?;

        for (info, path) in remove.into_iter() {
            println_info(info);
            match remove_dir_all(&path) {
                Ok(()) => {}
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::NotFound {
                        continue;
                    }
                    bail!("Failed to remove {}: {}", path.display(), e.to_string());
                }
            }
        }

        Ok(())
    } else {
        Ok(())
    }
}

pub fn self_update(force: bool) -> Result<()> {
    let download_cfg = DownloadCfg::new(
        component::FUELUP,
        TargetTriple::from_component(component::FUELUP)?,
        None,
    )?;

    let fuelup_bin = fuelup_bin();
    let fuelup_version = get_bin_version(&fuelup_bin).ok();

    if !force && fuelup_version >= Some(download_cfg.version.clone()) {
        info!(
            "Already up to date (fuelup v{})",
            download_cfg.version.to_string()
        );
        return Ok(());
    }

    let fuelup_new_dir = tempfile::tempdir()?;
    let fuelup_new_dir_path = fuelup_new_dir.path();
    let fuelup_backup = fuelup_new_dir_path.join("fuelup-backup");
    let fuelup_new = fuelup_new_dir_path.join(component::FUELUP);

    if let Err(e) = attempt_install_self(download_cfg, fuelup_new_dir_path) {
        // Skip all other steps if downloading fails here.
        // We do not need to handle failure, since this downloads to a tempdir.
        bail!("Failed to install fuelup: {}", e);
    };

    if fuelup_bin.exists() {
        if fuelup_backup.exists() {
            fs::remove_file(&fuelup_backup)?;
        };

        // Make a backup of fuelup within /tmp, in case download fails.
        if let Err(e) = fs::copy(&fuelup_bin, &fuelup_backup) {
            bail!(
                "Failed moving {} to {}: {}",
                &fuelup_bin.display(),
                &fuelup_backup.display(),
                e
            );
        }

        // Unlink the original 'fuelup', since we cannot write over a running executable.
        fs::remove_file(&fuelup_bin)?;
    }

    info!(
        "Moving {} to {}",
        fuelup_new.display(),
        &fuelup_bin.display()
    );
    if let Err(e) = fs::copy(&fuelup_new, &fuelup_bin) {
        error!(
            "Failed to replace old fuelup with new fuelup: {}. Attempting to restore backup fuelup.",
        e);
        // If we have failed to replace the old fuelup for whatever reason, we want the backup.
        // Although unlikely, should this last step fail, we will recommend to re-install fuelup
        // using fuelup-init.
        if let Err(e) = fs::copy(&fuelup_backup, &fuelup_bin) {
            bail!(
                "Could not restore backup fuelup. {}

You should re-install fuelup using fuelup-init:
`curl --proto '=https' --tlsv1.2 -sSf https://install.fuel.network/fuelup-init.sh | sh`",
                e
            );
        } else {
            let _ = fs::remove_file(&fuelup_backup);
        }

        bail!(
            "Old fuelup restored because something went wrong replacing old fuelup with new fuelup.",
        );
    } else {
        let fuelup_bin_dir = fuelup_bin_dir();
        let _ = fs::remove_file(&fuelup_new);
        for component in Components::collect_publishables()? {
            if fuelup_bin_dir.join(&component.name).exists() {
                hard_or_symlink_file(&fuelup_bin, &fuelup_bin_dir.join(component.name))?;
            }
        }
    };

    // Remove backup at the end.
    let _ = fs::remove_file(&fuelup_backup);

    Ok(())
}

#[cfg(test)]
mod test {
    use super::remove_path_from_content;
    use crate::path::FUELUP_DIR;

    #[test]
    fn dont_modify() {
        let content = "test\n\nbar\n";
        let (was_modified, _) = remove_path_from_content(content);
        assert!(!was_modified);
    }

    #[test]
    fn remove_only_path() {
        let content = format!("test\nPATH={}\nbar\n", FUELUP_DIR);
        let (was_modified, content) = remove_path_from_content(&content);
        assert!(was_modified);
        assert_eq!("test\nbar", content);
    }

    #[test]
    fn remove_subparts() {
        let content = format!("test\nPATH={}:foo\nbar\n", FUELUP_DIR);
        let (was_modified, content) = remove_path_from_content(&content);
        assert!(was_modified);
        assert_eq!("test\nPATH=foo\nbar", content);
    }
}
