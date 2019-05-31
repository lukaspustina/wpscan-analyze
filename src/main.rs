use clams;
use log;
use structopt;
use wpscan_analyze::{
    default_analysis, errors::*, AnalysisSummary, FromFile, OutputConfig, OutputDetail,
    OutputFormat, SanityCheck, Summary, WpScan, WpScanAnalysis,
};

use clams::prelude::*;
use std::path::{Path, PathBuf};
use std::process;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(
    name = "wpscan-analyze",
    about = "analyze wpscan json output and compares port states with specification",
    raw(setting = "structopt::clap::AppSettings::ColoredHelp")
)]
struct Args {
    /// wpscan json file
    #[structopt(short = "f", long = "wpscan", parse(from_os_str))]
    wpscan: PathBuf,
    /// Select output format
    #[structopt(
        short = "o",
        long = "output",
        default_value = "human",
        raw(possible_values = r#"&["human", "json", "none"]"#)
    )]
    output_format: OutputFormat,
    /// Select output detail level for human output; all or nok (not ok)
    #[structopt(
        long = "output-detail",
        default_value = "nok",
        raw(possible_values = r#"&["nok", "all"]"#)
    )]
    output_detail: OutputDetail,
    /// Do not use colored output
    #[structopt(long = "no-color")]
    no_color: bool,
    /// Silencium; use this for json output
    #[structopt(short = "s", long = "silent")]
    silent: bool,
    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[structopt(short = "v", long = "verbose", parse(from_occurrences))]
    verbosity: u64,
}

fn main() -> Result<()> {
    let args = Args::from_args();
    setup("wpscan_analyze", &args);
    debug!("args = {:#?}", args);

    let output_config = OutputConfig {
        detail: args.output_detail,
        format: args.output_format,
        color: !args.no_color,
    };

    run_wpscan_analyze(&args.wpscan, &output_config, args.silent).map(|code| process::exit(code))
}

fn setup(name: &str, args: &Args) {
    clams::console::set_color(!args.no_color);

    let level: Level = args.verbosity.into();
    if !args.silent {
        eprintln!(
            "{} version={}, log level={:?}",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION"),
            &level
        );
    }

    let log_config = LogConfig::new(
        std::io::stderr(),
        !args.no_color,
        Level(log::LevelFilter::Error),
        vec![ModLevel {
            module: name.to_owned(),
            level,
        }],
        None,
    );

    init_logging(log_config).expect("Failed to initialize logging");
}

fn run_wpscan_analyze<T: AsRef<Path>>(
    wpscan_file: T,
    output_config: &OutputConfig,
    silent: bool,
) -> Result<i32> {
    info!("Loading wpscan file");
    let wpscan = WpScan::from_file(wpscan_file.as_ref())?;
    info!("Checking wpscan results sanity");
    wpscan.is_sane()?;

    info!("Analyzing");
    let analyzer_result = default_analysis(&wpscan);
    debug!("{:#?}", analyzer_result);

    info!("Outputting results"); // Don't bail just because there is an output problem.
    if let Err(x) = output(output_config, &analyzer_result) {
        error!("Output failed because {}", x);
    }

    info!("Summarizing");
    if !silent {
        println!(
            "Analyzer result summary: {}={}, {}={}, {}={}",
            "outdated".yellow(),
            analyzer_result.outdated(),
            "vulnerabilities".red(),
            analyzer_result.vulnerabilities(),
            "failed".red(),
            analyzer_result.failed(),
        );
    }

    let summary = analyzer_result.summary();
    debug!("Summary={:?}", summary);
    let res = match summary {
        AnalysisSummary::Ok => 0,
        AnalysisSummary::Vulnerable => 11,
        AnalysisSummary::Outdated => 12,
        AnalysisSummary::Failed => 13,
    };

    Ok(res)
}

fn output(output_config: &OutputConfig, analyzer_result: &WpScanAnalysis) -> Result<usize> {
    match output_config.format {
        OutputFormat::Human => {
            use wpscan_analyze::output::HumanOutput;
            analyzer_result.output_tty(output_config)
        }
        OutputFormat::Json => {
            use wpscan_analyze::output::JsonOutput;
            let stdout = ::std::io::stdout();
            let mut writer = stdout.lock();
            analyzer_result.output(output_config, &mut writer)
        }
        OutputFormat::None => Ok(0),
    }
}
