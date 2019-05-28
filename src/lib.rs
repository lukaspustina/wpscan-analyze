pub mod analyze;
pub mod errors;
pub mod wpscan;
pub mod output;

pub use analyze::{default_analysis, AnalysisSummary, Summary, WpScanAnalysis};
pub use wpscan::WpScan;
pub use output::{OutputConfig, OutputDetail, OutputFormat};

use errors::*;

use failure::Fail;
use serde::de::{self, Deserialize, Deserializer};
use std::fmt::Display;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::str::FromStr;

pub trait FromFile {
    fn from_file<P: AsRef<Path>, E>(path: P) -> ::std::result::Result<Self, Error>
    where
        Self: Sized + FromStr<Err = E>,
        E: Fail
    {
        let contents = Self::string_from_file(path.as_ref())
            .map_err(|e|
                e.context(ErrorKind::InvalidFile(path.as_ref().to_string_lossy().into_owned())))?;

        Self::from_str(&contents)
            .map_err(|e|
                Error::from(ErrorKind::InvalidFormat))
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

fn from_str<'de, T, D>(deserializer: D) -> ::std::result::Result<T, D::Error>
where
    T: FromStr,
    T::Err: Display,
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    T::from_str(&s).map_err(de::Error::custom)
}
