use std::{cell::RefCell, path::PathBuf};
use toml_edit::{value, Document, Table, Value};

use anyhow::{bail, Result};

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

#[derive(Clone, Default)]
pub struct Settings {
    pub default_toolchain: Option<String>,
}

fn take_value(table: &mut Table, key: &str) -> Result<Value> {
    if let Some(i) = table.remove(key) {
        i.into_value()
            .or_else(|_| bail!("Item is None for key: '{}'", key))
    } else {
        bail!("missing key: '{}'", key)
    }
}

fn get_opt_string(table: &mut Table, key: &str) -> Result<Option<String>> {
    if let Ok(v) = take_value(table, key) {
        if let Some(s) = v.as_str() {
            Ok(Some(s.to_string()))
        } else {
            bail!("Expected string, got {}", key)
        }
    } else {
        Ok(None)
    }
}

impl Settings {
    pub(crate) fn from_toml(mut document: Document) -> Result<Self> {
        Ok(Self {
            default_toolchain: get_opt_string(document.as_table_mut(), "default_toolchain")?,
        })
    }

    pub(crate) fn parse(toml: &str) -> Result<Self> {
        let document = toml.parse::<Document>().expect("Invalid doc");
        Self::from_toml(document)
    }

    pub(crate) fn stringify(self) -> String {
        self.into_toml().to_string()
    }

    pub(crate) fn into_toml(self) -> Table {
        let mut table = Table::new();

        if let Some(v) = self.default_toolchain {
            table["default_toolchain"] = value(v);
        }

        table.fmt();
        table
    }
}
