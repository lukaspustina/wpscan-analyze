pub mod analyze;
pub mod errors;
pub mod output;
pub mod wpscan;

pub use analyze::{default_analysis, AnalysisSummary, Summary, WpScanAnalysis};
pub use output::{OutputConfig, OutputDetail, OutputFormat};
pub use wpscan::{FromFile, SanityCheck, WpScan};

#[cfg(test)]
mod tests {
    use crate::wpscan::WpScan;

    use std::str::FromStr;

    #[allow(non_snake_case)]
    pub(crate) fn WPSCAN_TEST_DATA() -> WpScan {
        let wp_file = include_str!("../tests/wpscan-example_com.json");

        WpScan::from_str(&wp_file).unwrap()
    }
}

