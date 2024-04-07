use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct AppInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub authors: String,
    pub license: String,
}

impl AppInfo {
    pub fn new() -> AppInfo {
        AppInfo {
            name: env!("CARGO_PKG_NAME").to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            description: env!("CARGO_PKG_DESCRIPTION").to_string(),
            authors: env!("CARGO_PKG_AUTHORS").to_string(),
            license: env!("CARGO_PKG_LICENSE").to_string(),
        }
    }
}

impl fmt::Display for AppInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if !self.name.is_empty() {
            write!(f, "Name: {}\n", self.name)?;
        }

        if !self.version.is_empty() {
            write!(f, "Version: {}\n", self.version)?;
        }

        if !self.description.is_empty() {
            write!(f, "Description: {}\n", self.description)?;
        }

        if !self.authors.is_empty() {
            write!(f, "Authors: {}\n", self.authors)?;
        }

        if !self.license.is_empty() {
            write!(f, "License: {}\n", self.license)?;
        }

        Ok(())
    }
}
