use crate::FromFile;
use crate::errors::*;

use serde::Deserialize;
use std::str::FromStr;
use failure::Fail;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct WpScan {
    banner: Banner,
    db_update_started: bool,
    db_update_finished: bool,
    start_time: usize,
    stop_time: usize,
    data_sent: usize,
    data_received: usize,
    target_url: String,
    effective_url: String,
    version: Version,
    main_theme: MainTheme,
    plugins: HashMap<String, Plugin>,
}

#[derive(Debug, Deserialize)]
pub struct Banner {
    version: String,
}

#[derive(Debug, Deserialize)]
pub struct Version {
    number: String,
    status: Option<String>,
    confidence: usize,
    vulnerabilities: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct MainTheme {
    latest_version: Option<String>,
    last_updated: Option<String>,
    outdated: bool,
    style_name: String,
    style_uri: String,
    vulnerabilities: serde_json::Value,
    version: Version,
}

#[derive(Debug, Deserialize)]
pub struct Plugin {
    slug: String,
    latest_version: Option<String>,
    last_updated: Option<String>,
    outdated: bool,
    vulnerabilities: serde_json::Value,
    version: Version,
}

impl FromStr for WpScan {
    type Err = Error;

    fn from_str(s: &str) -> ::std::result::Result<Self, Self::Err> {
        serde_json::from_str(s)
            .map_err(|e| e.context(ErrorKind::InvalidFormat).into())
    }
}

impl FromFile for WpScan {}

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