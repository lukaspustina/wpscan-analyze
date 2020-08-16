use crate::{
    analyze::{AnalysisSummary, AnalyzerResult, Summary, WpScanAnalysis},
    errors::*,
};

use failure::Fail;
use prettytable::{color, format, format::Alignment, Attr, Cell, Row, Table};
use serde_json;
use std::{io::Write, str::FromStr};
use crate::analyze::VersionState;

#[derive(Debug, PartialEq)]
pub enum OutputFormat {
    Human,
    Json,
    None,
}

impl FromStr for OutputFormat {
    type Err = Error;

    fn from_str(s: &str) -> ::std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_ref() {
            "human" => Ok(OutputFormat::Human),
            "json" => Ok(OutputFormat::Json),
            "none" => Ok(OutputFormat::None),
            _ => Err(ErrorKind::InvalidOutputFormat(s.to_string()).into()),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum OutputDetail {
    NotOkay,
    All,
}

impl FromStr for OutputDetail {
    type Err = Error;

    fn from_str(s: &str) -> ::std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_ref() {
            "nok" => Ok(OutputDetail::NotOkay),
            "all" => Ok(OutputDetail::All),
            _ => Err(ErrorKind::InvalidOutputDetail(s.to_string()).into()),
        }
    }
}

#[derive(Debug)]
pub struct OutputConfig {
    pub detail: OutputDetail,
    pub format: OutputFormat,
    pub color:  bool,
}

pub trait JsonOutput {
    fn output<T: Write>(&self, output_config: &OutputConfig, writer: &mut T) -> Result<usize>;
}

pub trait HumanOutput {
    fn output<T: Write>(&self, output_config: &OutputConfig, writer: &mut T) -> Result<usize>;
    fn output_tty(&self, output_config: &OutputConfig) -> Result<usize>;
}

impl<'a> JsonOutput for WpScanAnalysis<'a> {
    fn output<T: Write>(&self, _: &OutputConfig, writer: &mut T) -> Result<usize> {
        let json_str = serde_json::to_string(self).map_err(|e| e.context(ErrorKind::OutputFailed))?;
        let bytes = json_str.as_bytes();
        writer.write(bytes).map_err(|e| e.context(ErrorKind::OutputFailed))?;

        Ok(bytes.len())
    }
}

impl<'a> HumanOutput for WpScanAnalysis<'a> {
    fn output<T: Write>(&self, output_config: &OutputConfig, writer: &mut T) -> Result<usize> {
        self.build_table(output_config)
            .print(writer)
            .map_err(|e| e.context(ErrorKind::OutputFailed).into())
    }

    fn output_tty(&self, output_config: &OutputConfig) -> Result<usize> {
        if output_config.color {
            let len = self.build_table(output_config).printstd();
            Ok(len)
        } else {
            let stdout = ::std::io::stdout();
            let mut writer = stdout.lock();
            self.build_table(output_config)
                .print(&mut writer)
                .map_err(|e| e.context(ErrorKind::OutputFailed).into())
        }
    }
}

impl<'a> WpScanAnalysis<'a> {
    fn build_table(&self, output_config: &OutputConfig) -> Table {
        let mut table = Table::new();
        table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);

        table.set_titles(Row::new(vec![
            Cell::new("Component"),
            Cell::new("Version"),
            Cell::new("Version State"),
            Cell::new("Vulnerabilities"),
            Cell::new("Processing"),
            Cell::new("Result"),
        ]));

        table.add_row(result_to_row("WordPress", &self.word_press));
        table.add_row(result_to_row("Main Theme", &self.main_theme));
        for (k, v) in self.plugins.iter() {
            if output_config.detail == OutputDetail::NotOkay && v.summary() == AnalysisSummary::Ok {
                continue;
            }
            let text = format!("Plugin: {}", k);
            table.add_row(result_to_row(text.as_ref(), v));
        }

        table
    }
}

fn result_to_row(name: &str, result: &AnalyzerResult) -> Row {
    Row::new(vec![
        Cell::new(name),
        version_to_cell(result),
        match result.version_state() {
            VersionState::Latest =>
                Cell::new_align("Latest", Alignment::CENTER).with_style(Attr::ForegroundColor(color::GREEN)),
            VersionState::Outdated =>
                Cell::new_align("Outdated", Alignment::CENTER).with_style(Attr::ForegroundColor(color::YELLOW)),
            VersionState::Unknown =>
                Cell::new_align("Unknown", Alignment::CENTER).with_style(Attr::ForegroundColor(color::YELLOW)),
        },
        if result.vulnerabilities() > 0 {
            Cell::new_align(
                format!("{} vulnerabilities", result.vulnerabilities()).as_ref(),
                Alignment::CENTER,
            )
            .with_style(Attr::ForegroundColor(color::RED))
        } else {
            Cell::new_align("No vulnerabilities", Alignment::CENTER)
        },
        if result.failed() {
            Cell::new_align("Failed", Alignment::CENTER).with_style(Attr::ForegroundColor(color::RED))
        } else {
            Cell::new_align("Ok", Alignment::CENTER)
        },
        summary_to_cell(result),
    ])
}

fn version_to_cell(result: &AnalyzerResult) -> Cell {
    let text = match result.version() {
        Some(version) => version,
        None => "-",
    };

    Cell::new(text)
}

fn summary_to_cell(result: &AnalyzerResult) -> Cell {
    let mut cell = match result.summary() {
        AnalysisSummary::Ok => Cell::new("Ok"),
        AnalysisSummary::Outdated => Cell::new("Outdated").with_style(Attr::ForegroundColor(color::YELLOW)),
        AnalysisSummary::Unknown => Cell::new("Unknown").with_style(Attr::ForegroundColor(color::YELLOW)),
        AnalysisSummary::Vulnerable => Cell::new("Vulnerable").with_style(Attr::ForegroundColor(color::RED)),
        AnalysisSummary::Failed => Cell::new("Failed").with_style(Attr::ForegroundColor(color::RED)),
    };
    cell.align(Alignment::CENTER);

    cell
}

#[cfg(test)]
mod tests {
    use super::*;

    use spectral::prelude::*;

    #[test]
    fn output_format_from_str() {
        assert_that(&OutputFormat::from_str("human"))
            .is_ok()
            .is_equal_to(OutputFormat::Human);
        assert_that(&OutputFormat::from_str("json"))
            .is_ok()
            .is_equal_to(OutputFormat::Json);
        assert_that(&OutputFormat::from_str("none"))
            .is_ok()
            .is_equal_to(OutputFormat::None);
        assert_that(&OutputFormat::from_str("lukas")).is_err();
    }

    #[test]
    fn output_detail_from_str() {
        assert_that(&OutputDetail::from_str("all"))
            .is_ok()
            .is_equal_to(OutputDetail::All);
        assert_that(&OutputDetail::from_str("nok"))
            .is_ok()
            .is_equal_to(OutputDetail::NotOkay);
        assert_that(&OutputDetail::from_str("lukas")).is_err();
    }
}
