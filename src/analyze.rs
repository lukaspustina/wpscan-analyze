use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct AnalyzerResult<'a> {
    field: &'a str,
}