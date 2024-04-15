use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct AppInfo<'a> {
    pub name: &'a str,
    pub version: &'a str,
    pub authors: &'a str,
    pub description: &'a str,
    pub license: &'a str,
}

impl<'a> AppInfo<'a> {
    pub fn new(
        name: &'a str,
        version: &'a str,
        authors: &'a str,
        description: Option<&'a str>,
        license: Option<&'a str>,
    ) -> Self {
        Self {
            name,
            version,
            authors: authors,
            description: description.unwrap_or(""),
            license: license.unwrap_or(""),
        }
    }

    pub fn lib_info() -> Self {
        Self {
            name: env!("CARGO_PKG_NAME"),
            version: env!("CARGO_PKG_VERSION"),
            description: env!("CARGO_PKG_DESCRIPTION"),
            authors: env!("CARGO_PKG_AUTHORS"),
            license: env!("CARGO_PKG_LICENSE"),
        }
    }
}

impl<'a> fmt::Display for AppInfo<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} v{}", self.name, self.version)?;

        if !self.authors.is_empty() {
            write!(f, "\n{}", self.authors)?;
        }

        if !self.description.is_empty() {
            write!(f, "\n{}", self.description)?;
        }

        if !self.license.is_empty() {
            write!(f, "\nLicense: {}", self.license)?;
        }

        Ok(())
    }
}
