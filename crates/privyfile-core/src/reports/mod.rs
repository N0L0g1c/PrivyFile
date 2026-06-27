use std::fs;
use std::path::PathBuf;

use chrono::Utc;

use crate::types::{BatchItemResult, PrivyFileError, Result};

pub fn write_batch_report(results: &[BatchItemResult]) -> Result<String> {
    let timestamp = Utc::now().format("%Y%m%d-%H%M%S");
    let base_dir = std::env::temp_dir().join("privyfile-reports");
    fs::create_dir_all(&base_dir)?;

    let json_path = base_dir.join(format!("privyfile-report-{timestamp}.json"));
    let html_path = base_dir.join(format!("privyfile-report-{timestamp}.html"));

    let json = serde_json::to_string_pretty(results)
        .map_err(|error| PrivyFileError::Metadata(error.to_string()))?;
    fs::write(&json_path, &json)?;

    let html = render_html_report(results);
    fs::write(&html_path, html)?;

    Ok(json_path.to_string_lossy().into_owned())
}

pub fn render_html_report(results: &[BatchItemResult]) -> String {
    let rows = results
        .iter()
        .map(|item| {
            format!(
                "<tr><td>{}</td><td>{}</td><td>{}</td></tr>",
                html_escape(&item.path),
                if item.success { "OK" } else { "Failed" },
                html_escape(&item.message)
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8" />
  <title>PrivyFile Report</title>
  <style>
    body {{ font-family: Segoe UI, sans-serif; background: #0a0f1a; color: #e2e8f0; padding: 2rem; }}
    table {{ width: 100%; border-collapse: collapse; }}
    th, td {{ border: 1px solid #1e293b; padding: 0.75rem; text-align: left; }}
    th {{ background: #111827; color: #10b981; }}
  </style>
</head>
<body>
  <h1>PrivyFile Processing Report</h1>
  <p>Generated at {}</p>
  <table>
    <thead><tr><th>File</th><th>Status</th><th>Message</th></tr></thead>
    <tbody>{rows}</tbody>
  </table>
</body>
</html>"#,
        Utc::now().to_rfc3339()
    )
}

fn html_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

pub fn report_output_dir() -> PathBuf {
    std::env::temp_dir().join("privyfile-reports")
}
