use crate::analyze::{AnalyzerResult, Summary, AnalysisSummary, WpScanAnalysis};
use crate::errors::*;

use failure::Fail;
use prettytable::{
    Cell,
    Row,
    format::Alignment,
    color,
    color::Color,
    format,
    Attr,
    Table
};
use serde_json;
use std::io::Write;
use std::str::FromStr;

#[derive(Debug)]
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
    Fail,
    All,
}

impl FromStr for OutputDetail {
    type Err = Error;
    fn from_str(s: &str) -> ::std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_ref() {
            "fail" => Ok(OutputDetail::Fail),
            "all" => Ok(OutputDetail::All),
            _ => Err(ErrorKind::InvalidOutputDetail(s.to_string()).into()),
        }
    }
}

#[derive(Debug)]
pub struct OutputConfig {
    pub detail: OutputDetail,
    pub format: OutputFormat,
    pub color: bool,
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
        let json_str = serde_json::to_string(self)
            .map_err(|e| e.context(ErrorKind::OutputFailed))?;
        let bytes = json_str.as_bytes();
        writer
            .write(bytes)
            .map_err(|e| e.context(ErrorKind::OutputFailed))?;

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
        for (k,v) in self.plugins.iter() {
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
        if result.outdated() {
            Cell::new_align("Outdated", Alignment::CENTER)
                .with_style(Attr::ForegroundColor(color::YELLOW))
        } else {
            Cell::new_align("Latest", Alignment::CENTER)
                .with_style(Attr::ForegroundColor(color::GREEN))
        },
        if result.vulnerabilities() > 0 {
            Cell::new_align(format!("{} vulnerabilities", result.vulnerabilities()).as_ref(), Alignment::CENTER)
                .with_style(Attr::ForegroundColor(color::RED))
        } else {
            Cell::new_align("No vulnerabilities", Alignment::CENTER)
        },
        if result.failed() {
            Cell::new_align("Failed", Alignment::CENTER)
                .with_style(Attr::ForegroundColor(color::RED))
        } else {
            Cell::new_align("Ok", Alignment::CENTER)
        },
        summary_to_cell(result),
    ])
}

fn version_to_cell(result: &AnalyzerResult) -> Cell {
    let text = match result.version() {
       Some(version) => version,
       None => "-"
    };

    Cell::new(text)
}

fn summary_to_cell(result: &AnalyzerResult) -> Cell {
    let mut cell = match result.summary() {
        AnalysisSummary::Ok => Cell::new("Ok"),
        AnalysisSummary::Outdated => Cell::new("Outdated").with_style(Attr::ForegroundColor(color::YELLOW)),
        AnalysisSummary::Vulnerable => Cell::new("Vulnerable").with_style(Attr::ForegroundColor(color::RED)),
        AnalysisSummary::Failed => Cell::new("Failed").with_style(Attr::ForegroundColor(color::RED)),
    };
    cell.align(Alignment::CENTER);

    cell
}
