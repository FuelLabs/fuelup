use std::{cell::RefCell, path::PathBuf};

use anyhow::{anyhow, bail, Result};

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
        file::write_file(&self.path, &s.stringify())?;
        Ok(())
    }

    fn read_settings(&self) -> Result<()> {
        let mut needs_save = false;
        {
            let mut b = self.cache.borrow_mut();
            if b.is_none() {
                *b = Some(if file::is_file(&self.path) {
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

#[derive(Clone, Default)]
pub struct Settings {
    pub default_toolchain: Option<String>,
}

fn take_value(table: &mut toml::value::Table, key: &str, path: &str) -> Result<toml::Value> {
    table
        .remove(key)
        .ok_or_else(|| anyhow!(format!("missing key: '{}'", path.to_owned() + key)))
}

fn get_opt_string(table: &mut toml::value::Table, key: &str, path: &str) -> Result<Option<String>> {
    if let Ok(v) = take_value(table, key, path) {
        if let toml::Value::String(s) = v {
            Ok(Some(s))
        } else {
            bail!("Expected string, got {}", path.to_owned() + key)
        }
    } else {
        Ok(None)
    }
}

impl Settings {
    pub(crate) fn from_toml(mut table: toml::value::Table, path: &str) -> Result<Self> {
        Ok(Self {
            default_toolchain: get_opt_string(&mut table, "default_toolchain", path)?,
        })
    }

    pub(crate) fn parse(data: &str) -> Result<Self> {
        let value = toml::from_str(data)?;
        Self::from_toml(value, "")
    }

    pub(crate) fn stringify(self) -> String {
        toml::Value::Table(self.into_toml()).to_string()
    }

    pub(crate) fn into_toml(self) -> toml::value::Table {
        let mut result = toml::value::Table::new();

        if let Some(v) = self.default_toolchain {
            result.insert("default_toolchain".to_owned(), toml::Value::String(v));
        }

        result
    }
}
