use crate::errors::*;

use failure::Fail;
use log::warn;
use serde::Deserialize;
use serde::de::{self, Deserializer};
use std::collections::HashMap;
use std::fmt::Display;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::str::FromStr;


#[derive(Debug, Deserialize)]
pub struct WpScan {
    pub banner: Banner,
    pub db_update_started: bool,
    pub db_update_finished: bool,
    pub start_time: usize,
    pub stop_time: usize,
    pub data_sent: usize,
    pub data_received: usize,
    pub target_url: String,
    pub effective_url: String,
    #[serde(rename = "version")]
    pub word_press: Option<Version>,
    pub main_theme: Option<MainTheme>,
    pub plugins: HashMap<String, Plugin>,
}

#[derive(Debug, Deserialize)]
pub struct Banner {
    pub version: String,
}

#[derive(Debug, Deserialize)]
pub struct Version {
    pub number: String,
    pub status: Option<String>,
    pub confidence: usize,
    pub vulnerabilities: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct MainTheme {
    pub latest_version: Option<String>,
    pub last_updated: Option<String>,
    pub outdated: bool,
    pub style_name: String,
    pub style_uri: String,
    pub vulnerabilities: serde_json::Value,
    pub version: Version,
}

#[derive(Debug, Deserialize)]
pub struct Plugin {
    pub slug: String,
    pub latest_version: Option<String>,
    pub last_updated: Option<String>,
    pub outdated: bool,
    pub vulnerabilities: serde_json::Value,
    pub version: Option<Version>,
}

impl FromStr for WpScan {
    type Err = Error;

    fn from_str(s: &str) -> ::std::result::Result<Self, Self::Err> {
        serde_json::from_str(s).map_err(|e| e.context(ErrorKind::InvalidFormat).into())
    }
}

impl FromFile for WpScan {}

impl SanityCheck for WpScan {
    type Error = Error;

    fn is_sane(&self) -> std::result::Result<(), Self::Error> {
        if self.data_sent == 0 {
            return Err(Error::from(ErrorKind::InsaneWpScan(
                "no data has been sent.".to_string(),
            )));
        }
        if self.data_received == 0 {
            return Err(Error::from(ErrorKind::InsaneWpScan(
                "no data has been received.".to_string(),
            )));
        }
        if self.word_press.is_none() {
            return Err(Error::from(ErrorKind::InsaneWpScan(
                "WordPress version could not be recognized.".to_string(),
            )));
        }
        if self.main_theme.is_none() {
            return Err(Error::from(ErrorKind::InsaneWpScan(
                "Main theme could not be recognized.".to_string(),
            )));
        }
        if self.plugins.is_empty() {
            warn!("No plugins detected; this is okay, if you don't use plugins.");
        }

        Ok(())
    }
}

pub trait FromFile {
    fn from_file<P: AsRef<Path>, E>(path: P) -> ::std::result::Result<Self, Error>
    where
        Self: Sized + FromStr<Err = E>,
        E: Fail,
    {
        let contents = Self::string_from_file(path.as_ref()).map_err(|e| {
            e.context(ErrorKind::InvalidFile(
                path.as_ref().to_string_lossy().into_owned(),
            ))
        })?;

        Self::from_str(&contents).map_err(|_| Error::from(ErrorKind::InvalidFormat))
    }

    fn string_from_file<P: AsRef<Path>>(
        path: P,
    ) -> ::std::result::Result<String, ::std::io::Error> {
        let path: &Path = path.as_ref();

        let mut file = File::open(path)?;
        let mut contents = String::new();
        let _ = file.read_to_string(&mut contents)?;

        Ok(contents)
    }
}

pub trait SanityCheck {
    type Error;
    fn is_sane(&self) -> ::std::result::Result<(), Self::Error>;
}

pub(crate) fn from_str<'de, T, D>(deserializer: D) -> ::std::result::Result<T, D::Error>
where
    T: FromStr,
    T::Err: Display,
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    T::from_str(&s).map_err(de::Error::custom)
}

#[cfg(test)]
mod test {
    use super::WpScan;
    use crate::FromFile;

    use spectral::prelude::*;

    #[test]
    fn load_wpscan_results_file() -> () {
        let file = "tests/wpscan-example_com.json";

        let wp_scan = WpScan::from_file(file);

        assert_that(&wp_scan).is_ok();
        println!("{:#?}", wp_scan.unwrap());
    }
}
