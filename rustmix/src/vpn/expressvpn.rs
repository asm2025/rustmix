use execute::Execute;
use lazy_static::lazy_static;
use regex::Regex;
use std::process::{Command, Stdio};

use crate::{error::*, string::*, Result};

lazy_static! {
    static ref LOCATION_CLEAN: Regex = Regex::new(r"[\s\t]{2,}").unwrap();
}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ExpressVPNStatus {
    #[default]
    Unknown,
    NotActivated,
    Connected(Option<String>),
    Disconnected,
    Error(String),
}

#[derive(Debug)]
pub struct ExpressVPN;

impl ExpressVPN {
    const CMD: &'static str = "expressvpn";

    pub fn version(&self) -> Result<String> {
        let output = Command::new(Self::CMD)
            .arg("--version")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .execute_output()?;
        let ret = match output.status.code() {
            Some(ret) => ret,
            None => {
                return Err(CommandError(-1, String::from_utf8(output.stderr)?).into());
            }
        };

        if ret != 0 {
            return Err(CommandError(ret, String::from_utf8(output.stderr)?).into());
        }

        let text = String::from_utf8(output.stdout)?;
        let (_, n) = text
            .find_first(|c| c.is_digit(10))
            .ok_or(InvalidCommandResponseError)?;
        let text = text[n..].trim().to_string();
        Ok(text)
    }

    pub fn status(&self) -> Result<ExpressVPNStatus> {
        let output = Command::new(Self::CMD)
            .arg("status")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .execute_output()?;
        let ret = match output.status.code() {
            Some(ret) => ret,
            None => {
                return Err(CommandError(-1, String::from_utf8(output.stderr)?).into());
            }
        };

        if ret != 0 {
            return Err(CommandError(ret, String::from_utf8(output.stderr)?).into());
        }

        let text = String::from_utf8(output.stdout)?.trim().to_string();

        if text.contains("Not Activated") {
            return Ok(ExpressVPNStatus::NotActivated);
        }

        if text.contains("Not connected") {
            return Ok(ExpressVPNStatus::Disconnected);
        }

        if let Some(n) = text.find("Connected to") {
            let l = text[n..].find('\n').unwrap_or(text.len());
            let text = text[n..n + l].to_string();
            return Ok(ExpressVPNStatus::Connected(Some(text)));
        }

        if text.contains("connected") {
            return Ok(ExpressVPNStatus::Connected(None));
        }

        if text.contains("error") || text.contains("Oops!") || text.contains("problem") {
            return Ok(ExpressVPNStatus::Error(text));
        }

        Err(UnknownVPNResponseError(text).into())
    }

    pub fn connect(&self) -> Result<ExpressVPNStatus> {
        self.disconnect()?;
        self._connect(None)
    }

    pub fn connect_target(&self, value: &str) -> Result<ExpressVPNStatus> {
        self.disconnect()?;

        if value.is_empty() {
            self._connect(None)
        } else {
            self._connect(Some(value))
        }
    }

    fn _connect(&self, value: Option<&str>) -> Result<ExpressVPNStatus> {
        let mut command = Command::new(Self::CMD);
        command.arg("connect");

        if let Some(value) = value {
            if !value.is_empty() {
                command.arg(value);
            }
        }

        command.stdout(Stdio::piped()).stderr(Stdio::piped());

        let output = command.execute_output()?;
        let ret = match output.status.code() {
            Some(ret) => ret,
            None => {
                return Err(CommandError(-1, String::from_utf8(output.stderr)?).into());
            }
        };

        if ret != 0 {
            return Err(CommandError(ret, String::from_utf8(output.stderr)?).into());
        }

        let text = String::from_utf8(output.stdout)?.trim().to_string();
        match text.find("Connected to") {
            Some(n) => {
                let l = text[n..].find('\n').unwrap_or(text.len());
                let text = text[n..n + l].to_string();
                Ok(ExpressVPNStatus::Connected(Some(text)))
            }
            None => {
                if text.contains("Canceled") {
                    return Err(CanceledError.into());
                }

                if text.contains("unexpectedly") {
                    return Err(VPNError(text).into());
                }

                Err(UnknownVPNResponseError(text).into())
            }
        }
    }

    pub fn disconnect(&self) -> Result<ExpressVPNStatus> {
        let output = Command::new(Self::CMD)
            .arg("disconnect")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .execute_output()?;
        let ret = match output.status.code() {
            Some(ret) => ret,
            None => {
                return Err(CommandError(-1, String::from_utf8(output.stderr)?).into());
            }
        };

        if ret != 0 {
            let msg = String::from_utf8(output.stderr)?;

            if msg.contains("Disconnected") {
                return Ok(ExpressVPNStatus::Disconnected);
            }

            return Err(CommandError(ret, msg).into());
        }

        let text = String::from_utf8(output.stdout)?.trim().to_string();

        if text.contains("Disconnected") {
            return Ok(ExpressVPNStatus::Disconnected);
        }

        Err(UnknownVPNResponseError(text).into())
    }

    pub fn recent(&self) -> Result<Vec<String>> {
        let output = Command::new(Self::CMD)
            .arg("list")
            .arg("recent")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .execute_output()?;
        let ret = match output.status.code() {
            Some(ret) => ret,
            None => {
                return Err(CommandError(-1, String::from_utf8(output.stderr)?).into());
            }
        };

        if ret != 0 {
            return Err(CommandError(ret, String::from_utf8(output.stderr)?).into());
        }

        let text = String::from_utf8(output.stdout)?;
        Ok(self.locations_from_string(&text))
    }

    pub fn recommended(&self) -> Result<Vec<String>> {
        let output = Command::new(Self::CMD)
            .arg("list")
            .arg("recommended")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .execute_output()?;
        let ret = match output.status.code() {
            Some(ret) => ret,
            None => {
                return Err(CommandError(-1, String::from_utf8(output.stderr)?).into());
            }
        };

        if ret != 0 {
            return Err(CommandError(ret, String::from_utf8(output.stderr)?).into());
        }

        let text = String::from_utf8(output.stdout)?;
        Ok(self.locations_from_string(&text))
    }

    pub fn list(&self) -> Result<Vec<String>> {
        let output = Command::new(Self::CMD)
            .arg("list")
            .arg("all")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .execute_output()?;
        let ret = match output.status.code() {
            Some(ret) => ret,
            None => {
                return Err(CommandError(-1, String::from_utf8(output.stderr)?).into());
            }
        };

        if ret != 0 {
            return Err(CommandError(ret, String::from_utf8(output.stderr)?).into());
        }

        let text = String::from_utf8(output.stdout)?;
        Ok(self.locations_from_string(&text))
    }

    fn locations_from_string(&self, text: &str) -> Vec<String> {
        let mut locations = Vec::with_capacity(0);

        if text.is_empty() {
            return locations;
        }

        let n = text.find("-\n").unwrap_or(0);
        let l = text[n..].rfind("\n\n").unwrap_or(text.len());
        let text = text[n + 2..l].trim().to_string();

        for line in text.lines().filter(|line| !line.is_empty()) {
            let n = line.find(|c: char| c.is_whitespace()).unwrap_or(line.len());
            let line = line[..n].to_string();
            locations.push(line);
        }

        locations
    }

    pub fn refresh(&self) -> Result<()> {
        let output = Command::new(Self::CMD)
            .arg("refresh")
            .stderr(Stdio::piped())
            .execute_output()?;
        let ret = match output.status.code() {
            Some(ret) => ret,
            None => {
                return Err(CommandError(-1, String::from_utf8(output.stderr)?).into());
            }
        };

        if ret != 0 {
            return Err(CommandError(ret, String::from_utf8(output.stderr)?).into());
        }

        Ok(())
    }

    pub fn network_lock(&self, enable: bool) -> Result<()> {
        let output = Command::new(Self::CMD)
            .arg("preferences")
            .arg("set")
            .arg("network_lock")
            .arg(if enable { "on" } else { "off" })
            .stderr(Stdio::piped())
            .execute_output()?;
        let ret = match output.status.code() {
            Some(ret) => ret,
            None => {
                return Err(CommandError(-1, String::from_utf8(output.stderr)?).into());
            }
        };

        if ret != 0 {
            return Err(CommandError(ret, String::from_utf8(output.stderr)?).into());
        }

        Ok(())
    }

    pub fn block_trackers(&self, enable: bool) -> Result<()> {
        let output = Command::new(Self::CMD)
            .arg("preferences")
            .arg("set")
            .arg("block_trackers")
            .arg(if enable { "true" } else { "false" })
            .stderr(Stdio::piped())
            .execute_output()?;
        let ret = match output.status.code() {
            Some(ret) => ret,
            None => {
                return Err(CommandError(-1, String::from_utf8(output.stderr)?).into());
            }
        };

        if ret != 0 {
            return Err(CommandError(ret, String::from_utf8(output.stderr)?).into());
        }

        Ok(())
    }
}
