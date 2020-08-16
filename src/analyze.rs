use crate::wpscan::{Plugin, WpScan};

use serde::Serialize;
use std::{collections::HashMap, hash::BuildHasher};

#[derive(Debug, Serialize, PartialEq, Clone, Copy)]
pub enum VersionState {
    Latest,
    Outdated,
    Unknown,
}

impl VersionState {
    pub fn is_latest(&self) -> bool {
        *self == VersionState::Latest
    }

    pub fn is_outdated(&self) -> bool {
        *self == VersionState::Outdated
    }

    pub fn is_unknown(&self) -> bool {
        *self == VersionState::Unknown
    }

}

#[derive(Debug, Serialize, PartialEq, Clone)]
pub struct Analysis<'a> {
    pub version:         &'a str,
    pub version_state:   VersionState,
    pub vulnerabilities: usize,
}

#[derive(Debug, Serialize, PartialEq, Clone)]
pub enum AnalyzerResult<'a> {
    Success(Analysis<'a>),
    Failed(String),
}

impl<'a> AnalyzerResult<'a> {
    pub fn version(&self) -> Option<&'a str> {
        match self {
            AnalyzerResult::Success(analysis) => Some(analysis.version),
            AnalyzerResult::Failed(_) => None,
        }
    }

    pub fn vulnerabilities(&self) -> usize {
        match self {
            AnalyzerResult::Success(analysis) => analysis.vulnerabilities,
            AnalyzerResult::Failed(_) => 0,
        }
    }

    pub fn version_state(&self) -> VersionState {
        match self {
            AnalyzerResult::Success(analysis) => analysis.version_state,
            AnalyzerResult::Failed(_) => VersionState::Unknown,
        }
    }

    pub fn failed(&self) -> bool {
        match self {
            AnalyzerResult::Success(_) => false,
            AnalyzerResult::Failed(_) => true,
        }
    }
}

#[derive(Debug, Serialize, PartialEq, Clone)]
pub struct WpScanAnalysis<'a> {
    pub word_press: AnalyzerResult<'a>,
    pub main_theme: AnalyzerResult<'a>,
    pub plugins:    HashMap<&'a str, AnalyzerResult<'a>>,
}

impl<'a> WpScanAnalysis<'a> {
    pub fn vulnerabilities(&self) -> usize {
        self.word_press.vulnerabilities()
            + self.main_theme.vulnerabilities()
            + self
                .plugins
                .values()
                .map(AnalyzerResult::vulnerabilities)
                .sum::<usize>()
    }

    pub fn outdated(&self) -> usize {
        self.plugins.values().filter(|x| x.version_state().is_outdated()).count()
            + if self.word_press.version_state().is_outdated() { 1 } else { 0 }
            + if self.main_theme.version_state().is_outdated() { 1 } else { 0 }
    }

    pub fn unknown(&self) -> usize {
        self.plugins.values().filter(|x| x.version_state().is_unknown() ).count()
            + if self.word_press.version_state().is_unknown() { 1 } else { 0 }
            + if self.main_theme.version_state().is_unknown() { 1 } else { 0 }
    }

    pub fn failed(&self) -> usize {
        self.plugins.values().filter(|x| x.failed()).count()
            + if self.word_press.failed() { 1 } else { 0 }
            + if self.main_theme.failed() { 1 } else { 0 }
    }
}

pub fn default_analysis(wpscan: &WpScan) -> WpScanAnalysis {
    let analyzer = DefaultAnalyzer::new(wpscan);
    analyzer.analyze()
}

pub trait Analyzer<'a> {
    fn analyze(&self) -> WpScanAnalysis<'a>;
}

pub struct DefaultAnalyzer<'a> {
    wpscan: &'a WpScan,
}

impl<'a> DefaultAnalyzer<'a> {
    pub fn new(wpscan: &'a WpScan) -> DefaultAnalyzer<'a> { DefaultAnalyzer { wpscan } }
}

impl<'a> Analyzer<'a> for DefaultAnalyzer<'a> {
    fn analyze(&self) -> WpScanAnalysis<'a> {
        let word_press = self.analyze_word_press();
        let main_theme = self.analyze_main_theme();
        let plugins: HashMap<&str, AnalyzerResult> = self.analyze_plugins();

        WpScanAnalysis {
            word_press,
            main_theme,
            plugins,
        }
    }
}

impl<'a> DefaultAnalyzer<'a> {
    fn analyze_word_press(&self) -> AnalyzerResult<'a> {
        if self.wpscan.word_press.is_none() {
            return AnalyzerResult::Success(Analysis {
                version: "-",
                version_state: VersionState::Unknown,
                vulnerabilities: 0,
            })
        };
        let word_press = self.wpscan.word_press.as_ref().unwrap(); // Safe

        let version_state = match word_press.status {
            Some(ref status) if status == "latest" => VersionState::Latest,
            Some(_) => VersionState::Outdated,
            _ => VersionState::Unknown,
        };

        let vulnerabilities = if let Some(serde_json::Value::Array(list)) = &word_press.vulnerabilities {
            list.len()
        } else {
            0
        };

        AnalyzerResult::Success(Analysis {
            version: &word_press.number,
            version_state,
            vulnerabilities,
        })
    }

    fn analyze_main_theme(&self) -> AnalyzerResult<'a> {
        if self.wpscan.main_theme.is_none() {
            return AnalyzerResult::Success(Analysis {
                version: "-",
                version_state: VersionState::Unknown,
                vulnerabilities: 0,
            })
        };
        let main_theme = self.wpscan.main_theme.as_ref().unwrap(); // Safe

        let version_state = match main_theme.outdated {
            Some(false) => VersionState::Latest,
            Some(true) if main_theme.version.is_none() => VersionState::Unknown,
            Some(true) => VersionState::Outdated,
            _ => VersionState::Unknown,
        };

        let vulnerabilities = if let Some(serde_json::Value::Array(list)) = &main_theme.vulnerabilities {
            list.len()
        } else {
            0
        };

        AnalyzerResult::Success(Analysis {
            version: &main_theme
                .version
                .as_ref()
                .map(|x| x.number.as_ref())
                .unwrap_or_else(|| "-"),
            version_state,
            vulnerabilities,
        })
    }

    fn analyze_plugins(&self) -> HashMap<&'a str, AnalyzerResult<'a>> {
        self.wpscan
            .plugins
            .iter()
            .map(|(k, v)| (k.as_str(), DefaultAnalyzer::analyze_plugin(v)))
            .collect()
    }

    fn analyze_plugin(plugin: &'a Plugin) -> AnalyzerResult<'a> {
        let version_state = match plugin.outdated {
            Some(false) => VersionState::Latest,
            Some(true) if plugin.version.is_none() => VersionState::Unknown,
            Some(true) => VersionState::Outdated,
            _ => VersionState::Unknown,
        };

        let vulnerabilities = if let Some(serde_json::Value::Array(list)) = &plugin.vulnerabilities {
            list.len()
        } else {
            0
        };

        AnalyzerResult::Success(Analysis {
            version: plugin
                .version
                .as_ref()
                .map(|x| x.number.as_ref())
                .unwrap_or_else(|| "-"),
            version_state,
            vulnerabilities,
        })
    }
}

#[derive(Debug, Serialize, PartialEq)]
pub enum AnalysisSummary {
    Outdated,
    Unknown,
    Vulnerable,
    Failed,
    Ok,
}

pub trait Summary {
    fn summary(&self) -> AnalysisSummary;
}

impl<'a> Summary for AnalyzerResult<'a> {
    fn summary(&self) -> AnalysisSummary {
        match self {
            AnalyzerResult::Success(analysis) if analysis.vulnerabilities > 0 => AnalysisSummary::Vulnerable,
            AnalyzerResult::Success(analysis) if analysis.version_state.is_outdated() => AnalysisSummary::Outdated,
            AnalyzerResult::Success(analysis) if analysis.version_state.is_unknown() => AnalysisSummary::Unknown,
            AnalyzerResult::Failed(_) => AnalysisSummary::Failed,
            _ => AnalysisSummary::Ok,
        }
    }
}

impl<'a, S: BuildHasher> Summary for HashMap<&'a str, AnalyzerResult<'a>, S> {
    fn summary(&self) -> AnalysisSummary {
        let (success, fails): (Vec<_>, Vec<_>) = self.values().partition(|x| {
            match x {
                AnalyzerResult::Success(_) => true,
                AnalyzerResult::Failed(_) => false,
            }
        });
        if !fails.is_empty() {
            return AnalysisSummary::Failed;
        }

        let vulnerabilities = success.iter().fold(0, |acc, x| {
            acc + match x {
                AnalyzerResult::Success(analysis) => analysis.vulnerabilities,
                AnalyzerResult::Failed(_) => 0,
            }
        });
        if vulnerabilities > 0 {
            return AnalysisSummary::Vulnerable;
        }

        let outdated = success.iter().fold(0, |acc, x| {
            acc + match x {
                AnalyzerResult::Success(analysis) if analysis.version_state.is_outdated() => 1,
                _ => 0,
            }
        });
        if outdated > 0 {
            return AnalysisSummary::Outdated;
        }

        let unknown = success.iter().fold(0, |acc, x| {
            acc + match x {
                AnalyzerResult::Success(analysis) if analysis.version_state.is_unknown() => 1,
                _ => 0,
            }
        });
        if unknown > 0 {
            return AnalysisSummary::Unknown;
        }

        AnalysisSummary::Ok
    }
}

impl<'a> Summary for WpScanAnalysis<'a> {
    fn summary(&self) -> AnalysisSummary {
        let summaries = &[
            self.word_press.summary(),
            self.main_theme.summary(),
            self.plugins.summary(),
        ];

        if summaries.contains(&AnalysisSummary::Vulnerable) {
            return AnalysisSummary::Vulnerable;
        }
        if summaries.contains(&AnalysisSummary::Outdated) {
            return AnalysisSummary::Outdated;
        }
        if summaries.contains(&AnalysisSummary::Failed) {
            return AnalysisSummary::Failed;
        }

        AnalysisSummary::Ok
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::{WPSCAN_TEST_DATA, WPSCAN_TEST_DATA_NO_WP_VERSION};

    use spectral::prelude::*;

    #[test]
    fn default_analysis_stats() {
        let wpscan = WPSCAN_TEST_DATA();

        let analyzer = DefaultAnalyzer::new(&wpscan);
        let analysis = analyzer.analyze();

        asserting("vulnerabilities")
            .that(&analysis.vulnerabilities())
            .is_equal_to(1);
        asserting("outdated").that(&analysis.outdated()).is_equal_to(2);
        asserting("unknown").that(&analysis.unknown()).is_equal_to(1);
        asserting("failed").that(&analysis.failed()).is_equal_to(0);
    }

    #[test]
    fn default_analysis() {
        let expected_plugins: HashMap<&str, AnalyzerResult> = [
            (
                "jm-twitter-cards",
                AnalyzerResult::Success(Analysis {
                    version:         "9.4",
                    version_state:        VersionState::Outdated,
                    vulnerabilities: 0,
                }),
            ),
            (
                "js_composer",
                AnalyzerResult::Success(Analysis {
                    version:         "4.11.1",
                    version_state:        VersionState::Latest,
                    vulnerabilities: 0,
                }),
            ),
            (
                "wordpress-seo",
                AnalyzerResult::Success(Analysis {
                    version:         "8.0",
                    version_state:        VersionState::Outdated,
                    vulnerabilities: 1,
                }),
            ),
            (
                "bwp-minify",
                AnalyzerResult::Success(Analysis {
                    version:         "1.3.3",
                    version_state:        VersionState::Latest,
                    vulnerabilities: 0,
                }),
            ),
            (
                "wp-super-cache",
                AnalyzerResult::Success(Analysis {
                    version:         "-",
                    version_state:        VersionState::Unknown,
                    vulnerabilities: 0,
                }),
            ),
        ]
        .iter()
        .cloned()
        .collect();
        let expected = WpScanAnalysis {
            word_press: AnalyzerResult::Success(Analysis {
                version:         "4.9.10",
                version_state:        VersionState::Latest,
                vulnerabilities: 0,
            }),
            main_theme: AnalyzerResult::Success(Analysis {
                version:         "3.2.1",
                version_state:        VersionState::Latest,
                vulnerabilities: 0,
            }),
            plugins:    expected_plugins,
        };

        let wpscan = WPSCAN_TEST_DATA();

        let analyzer = DefaultAnalyzer::new(&wpscan);
        let analysis = analyzer.analyze();

        asserting("Default Analysis").that(&analysis).is_equal_to(expected);
    }

    #[test]
    fn default_analysis_summary_success() {
        let result = AnalyzerResult::Success(Analysis {
            version:         "4.9.10",
            version_state:        VersionState::Latest,
            vulnerabilities: 0,
        });

        asserting("Ok").that(&result.summary()).is_equal_to(AnalysisSummary::Ok);
    }

    #[test]
    fn default_analysis_summary_vulnaribility() {
        let result = AnalyzerResult::Success(Analysis {
            version:         "4.9.10",
            version_state:        VersionState::Latest,
            vulnerabilities: 1,
        });

        asserting("Ok")
            .that(&result.summary())
            .is_equal_to(AnalysisSummary::Vulnerable);
    }

    #[test]
    fn default_analysis_summary_outdated() {
        let result = AnalyzerResult::Success(Analysis {
            version:         "4.9.10",
            version_state:        VersionState::Outdated,
            vulnerabilities: 0,
        });

        asserting("Ok")
            .that(&result.summary())
            .is_equal_to(AnalysisSummary::Outdated);
    }

    #[test]
    fn default_analysis_summary_failed() {
        let result = AnalyzerResult::Failed("Failure reason".to_string());

        asserting("Ok")
            .that(&result.summary())
            .is_equal_to(AnalysisSummary::Failed);
    }


    #[test]
    fn analysis_no_wp_version() {
        let wpscan = WPSCAN_TEST_DATA_NO_WP_VERSION();

        let analyzer = DefaultAnalyzer::new(&wpscan);
        let analysis = analyzer.analyze();

        asserting("vulnerabilities")
            .that(&analysis.vulnerabilities())
            .is_equal_to(1);
        asserting("outdated").that(&analysis.outdated()).is_equal_to(2);
        asserting("unknown").that(&analysis.unknown()).is_equal_to(2);
        asserting("failed").that(&analysis.failed()).is_equal_to(0);
    }
}
