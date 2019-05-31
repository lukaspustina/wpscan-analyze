use crate::wpscan::{Plugin, WpScan};

use serde::Serialize;
use std::collections::HashMap;
use std::hash::BuildHasher;

#[derive(Debug, Serialize)]
pub struct Analysis<'a> {
    pub version: &'a str,
    pub outdated: bool,
    pub vulnerabilities: usize,
}

#[derive(Debug, Serialize)]
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

    pub fn outdated(&self) -> bool {
        match self {
            AnalyzerResult::Success(analysis) => analysis.outdated,
            AnalyzerResult::Failed(_) => false,
        }
    }

    pub fn failed(&self) -> bool {
        match self {
            AnalyzerResult::Success(_) => false,
            AnalyzerResult::Failed(_) => true,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct WpScanAnalysis<'a> {
    pub word_press: AnalyzerResult<'a>,
    pub main_theme: AnalyzerResult<'a>,
    pub plugins: HashMap<&'a str, AnalyzerResult<'a>>,
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
        self.plugins.values().filter(|x| x.outdated()).count()
            + if self.word_press.outdated() { 1 } else { 0 }
            + if self.main_theme.outdated() { 1 } else { 0 }
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
    pub fn new(wpscan: &'a WpScan) -> DefaultAnalyzer<'a> {
        DefaultAnalyzer { wpscan }
    }
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
            return AnalyzerResult::Failed("Could not determine WordPress".to_string());
        };
        let word_press = self.wpscan.word_press.as_ref().unwrap(); // Safe

        let outdated = if let Some(status) = &word_press.status {
            status != "latest"
        } else {
            return AnalyzerResult::Failed("Could not determine version status".to_string());
        };

        let vulnerabilities = if let serde_json::Value::Array(list) = &word_press.vulnerabilities {
            list.len()
        } else {
            0
        };

        AnalyzerResult::Success(Analysis {
            version: &word_press.number,
            outdated,
            vulnerabilities,
        })
    }

    fn analyze_main_theme(&self) -> AnalyzerResult<'a> {
        if self.wpscan.main_theme.is_none() {
            return AnalyzerResult::Failed("Could not determine main theme".to_string());
        };
        let main_theme = self.wpscan.main_theme.as_ref().unwrap(); // Safe

        let outdated = main_theme.outdated;

        let vulnerabilities = if let serde_json::Value::Array(list) = &main_theme.vulnerabilities {
            list.len()
        } else {
            0
        };

        AnalyzerResult::Success(Analysis {
            version: &main_theme.version.number,
            outdated,
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
        let outdated = plugin.outdated;

        let vulnerabilities = if let serde_json::Value::Array(list) = &plugin.vulnerabilities {
            list.len()
        } else {
            0
        };

        AnalyzerResult::Success(Analysis {
            version: &plugin.version.number,
            outdated,
            vulnerabilities,
        })
    }
}

#[derive(Debug, Serialize, PartialEq)]
pub enum AnalysisSummary {
    Outdated,
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
            AnalyzerResult::Success(analysis) if analysis.vulnerabilities > 0 => {
                AnalysisSummary::Vulnerable
            }
            AnalyzerResult::Success(analysis) if analysis.outdated => AnalysisSummary::Outdated,
            AnalyzerResult::Failed(_) => AnalysisSummary::Failed,
            _ => AnalysisSummary::Ok,
        }
    }
}

impl<'a, S: BuildHasher> Summary for HashMap<&'a str, AnalyzerResult<'a>, S> {
    fn summary(&self) -> AnalysisSummary {
        let (success, fails): (Vec<_>, Vec<_>) = self.values().partition(|x| match x {
            AnalyzerResult::Success(_) => true,
            AnalyzerResult::Failed(_) => false,
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
                AnalyzerResult::Success(analysis) if analysis.outdated => 1,
                _ => 0,
            }
        });
        if outdated > 0 {
            return AnalysisSummary::Outdated;
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
