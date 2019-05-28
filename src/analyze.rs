use crate::wpscan::{WpScan, Plugin};

use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Serialize)]
pub struct Analysis {
    outdated: bool,
    vulnerabilities: usize,
}

#[derive(Debug, Serialize)]
pub enum AnalyzerResult {
    Success(Analysis),
    Failed(String),
}

#[derive(Debug, Serialize)]
pub struct WpScanAnalysis<'a> {
    word_press: AnalyzerResult,
    main_theme: AnalyzerResult,
    plugins: HashMap<&'a str, AnalyzerResult>,
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
        DefaultAnalyzer {
            wpscan
        }
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
    fn analyze_word_press(&self) -> AnalyzerResult {
        if self.wpscan.word_press.is_none() {
            return AnalyzerResult::Failed("Could not determine WordPress".to_string())
        };
        let word_press = self.wpscan.word_press.as_ref().unwrap(); // Safe

        let outdated = if let Some(status) = &word_press.status {
            status != "latest"
        } else {
            return AnalyzerResult::Failed("Could not determine version status".to_string())
        };

        let vulnerabilities = if let serde_json::Value::Array(list) = &word_press.vulnerabilities {
            list.len()
        } else {
            0
        };

        AnalyzerResult::Success(
            Analysis {
                outdated,
                vulnerabilities,
            }
        )
    }

    fn analyze_main_theme(&self) -> AnalyzerResult {
        if self.wpscan.main_theme.is_none() {
            return AnalyzerResult::Failed("Could not determine main theme".to_string())
        };
        let main_theme= self.wpscan.main_theme.as_ref().unwrap(); // Safe

        let outdated = main_theme.outdated;

        let vulnerabilities = if let serde_json::Value::Array(list) = &main_theme.vulnerabilities {
            list.len()
        } else {
            0
        };

        AnalyzerResult::Success(
            Analysis {
                outdated,
                vulnerabilities,
            }
        )
    }

    fn analyze_plugins(&self) -> HashMap<&'a str, AnalyzerResult> {
        self.wpscan.plugins.iter()
            .map(|(k,v)| (k.as_str(), DefaultAnalyzer::analyze_plugin(v)))
            .collect()
    }

    fn analyze_plugin(plugin: &Plugin) -> AnalyzerResult {
        let outdated = plugin.outdated;

        let vulnerabilities = if let serde_json::Value::Array(list) = &plugin.vulnerabilities {
            list.len()
        } else {
            0
        };

        AnalyzerResult::Success(
            Analysis {
                outdated,
                vulnerabilities,
            }
        )
    }
}