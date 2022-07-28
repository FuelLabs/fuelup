use serde::{Deserialize, Serialize};
use std::{cell::RefCell, path::PathBuf};
use toml_edit::{de, ser, Document};

use anyhow::Result;

use crate::file;

pub struct SettingsFile {
    path: PathBuf,
    cache: RefCell<Option<Settings>>,
}

impl SettingsFile {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            cache: RefCell::new(None),
        }
    }

    fn write_settings(&self) -> Result<()> {
        let s = self.cache.borrow().as_ref().unwrap().clone();
        file::write_file(&self.path, &s.stringify()?)?;
        Ok(())
    }

    fn read_settings(&self) -> Result<()> {
        let mut needs_save = false;
        {
            let mut b = self.cache.borrow_mut();
            if b.is_none() {
                *b = Some(if self.path.is_file() {
                    let content = file::read_file("settings", &self.path)?;
                    Settings::parse(&content)?
                } else {
                    needs_save = true;
                    Default::default()
                });
            }
        }
        if needs_save {
            self.write_settings()?;
        }
        Ok(())
    }

    pub fn with<T, F: FnOnce(&Settings) -> Result<T>>(&self, f: F) -> Result<T> {
        self.read_settings()?;

        // Settings can no longer be None so it's OK to unwrap
        f(self.cache.borrow().as_ref().unwrap())
    }

    pub(crate) fn with_mut<T, F: FnOnce(&mut Settings) -> Result<T>>(&self, f: F) -> Result<T> {
        self.read_settings()?;

        // Settings can no longer be None so it's OK to unwrap
        let result = { f(self.cache.borrow_mut().as_mut().unwrap())? };
        self.write_settings()?;
        Ok(result)
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Settings {
    pub default_toolchain: Option<String>,
}

impl Settings {
    pub(crate) fn parse(toml: &str) -> Result<Self> {
        let settings: Settings = de::from_str(toml)?;
        Ok(settings)
    }

    pub(crate) fn stringify(self) -> Result<String> {
        Ok(self.into_toml()?.to_string())
    }

    pub(crate) fn into_toml(self) -> std::result::Result<Document, ser::Error> {
        ser::to_document(&self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::file::read_file;

    pub(crate) fn with_mock_fuelup_dir<F>(f: F) -> Result<()>
    where
        F: FnOnce(tempfile::TempDir) -> Result<()>,
    {
        let mock_fuelup_dir = tempfile::tempdir()?;
        f(mock_fuelup_dir)
    }

    #[test]
    fn parse_settings() {
        let settings_path = std::env::current_dir()
            .unwrap()
            .join("tests/settings-example.toml");

        let settings_file = read_file("settings-example", &settings_path).unwrap();
        let settings = Settings::parse(&settings_file).unwrap();

        assert_eq!(
            settings.default_toolchain.unwrap(),
            "latest-x86_64-apple-darwin"
        )
    }

    #[test]
    fn write_settings() -> Result<()> {
        with_mock_fuelup_dir(|dir| {
            let settings_path = dir.path().join("settings-example-dst.toml");

            let settings_file = SettingsFile::new(PathBuf::from(&settings_path));
            let new_default_toolchain = String::from("new-default-toolchain");

            settings_file.with_mut(|s| {
                s.default_toolchain = Some(new_default_toolchain.clone());
                Ok(())
            })?;

            let settings =
                Settings::parse(&read_file("settings-example-dst", &settings_path).unwrap())
                    .unwrap();
            assert_eq!(settings.default_toolchain.unwrap(), new_default_toolchain);
            Ok(())
        })
    }

    #[test]
    fn stringify_settings() {
        let expected_toml = r#"default_toolchain = "yet-another-default-toolchain"
"#;

        let settings = Settings {
            default_toolchain: Some("yet-another-default-toolchain".to_string()),
        };

        let stringified = settings.stringify().unwrap();
        assert_eq!(stringified, expected_toml);
    }
}
