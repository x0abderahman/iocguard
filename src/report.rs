//! Module for report generation (CSV, JSON, and summary files).

use std::fs;
use std::io::Write;
use std::path::Path;

use crate::domain::{DomainRecord, DomainStatus};

/// Generate report files in the output directory.
///
/// If `json_output` is true, generates `report.json` instead of CSV files.
/// Always generates `summary.txt`.
pub fn generate_report(
    records: &[DomainRecord],
    out_dir: &Path,
    allowlisted_count: usize,
    json_output: bool,
) -> Result<(), String> {
    // Create output directory
    fs::create_dir_all(out_dir).map_err(|e| {
        format!(
            "Failed to create output directory '{}': {}",
            out_dir.display(),
            e
        )
    })?;

    // Count statistics
    let total = records.len();
    let valid_count = records
        .iter()
        .filter(|r| r.status == DomainStatus::Accepted)
        .count();
    let suspicious_count = records
        .iter()
        .filter(|r| r.status == DomainStatus::Suspicious)
        .count();
    let invalid_count = records
        .iter()
        .filter(|r| r.status == DomainStatus::Rejected)
        .count();

    if json_output {
        write_json_report(
            records,
            out_dir,
            total,
            valid_count,
            invalid_count,
            suspicious_count,
            allowlisted_count,
        )?;
    } else {
        // Write accepted.csv
        let accepted: Vec<&DomainRecord> = records
            .iter()
            .filter(|r| r.status == DomainStatus::Accepted)
            .collect();
        write_csv(
            out_dir.join("accepted.csv"),
            &["normalized_domain", "source", "status", "reason"],
            &accepted,
        )?;

        // Write suspicious.csv
        let suspicious: Vec<&DomainRecord> = records
            .iter()
            .filter(|r| r.status == DomainStatus::Suspicious)
            .collect();
        write_csv(
            out_dir.join("suspicious.csv"),
            &["normalized_domain", "source", "status", "reason"],
            &suspicious,
        )?;

        // Write rejected.csv (include original_value)
        let rejected: Vec<&DomainRecord> = records
            .iter()
            .filter(|r| r.status == DomainStatus::Rejected)
            .collect();
        write_rejected_csv(out_dir.join("rejected.csv"), &rejected)?;
    }

    // Write summary.txt
    write_summary(
        out_dir.join("summary.txt"),
        total,
        valid_count,
        invalid_count,
        suspicious_count,
        allowlisted_count,
    )?;

    Ok(())
}

/// Write JSON report with all data.
fn write_json_report(
    records: &[DomainRecord],
    out_dir: &Path,
    total: usize,
    valid: usize,
    invalid: usize,
    suspicious: usize,
    allowlisted: usize,
) -> Result<(), String> {
    let path = out_dir.join("report.json");
    let mut file = fs::File::create(&path)
        .map_err(|e| format!("Failed to create '{}': {}", path.display(), e))?;

    writeln!(file, "{{").map_err(|e| format!("Write error: {}", e))?;
    writeln!(file, "  \"summary\": {{").map_err(|e| format!("Write error: {}", e))?;
    writeln!(file, "    \"total_lines_processed\": {},", total)
        .map_err(|e| format!("Write error: {}", e))?;
    writeln!(file, "    \"valid_domains\": {},", valid)
        .map_err(|e| format!("Write error: {}", e))?;
    writeln!(file, "    \"invalid_domains\": {},", invalid)
        .map_err(|e| format!("Write error: {}", e))?;
    writeln!(file, "    \"suspicious_domains\": {},", suspicious)
        .map_err(|e| format!("Write error: {}", e))?;
    writeln!(file, "    \"allowlisted_domains\": {}", allowlisted)
        .map_err(|e| format!("Write error: {}", e))?;
    writeln!(file, "  }},").map_err(|e| format!("Write error: {}", e))?;

    // Accepted records
    writeln!(file, "  \"accepted\": [").map_err(|e| format!("Write error: {}", e))?;
    let accepted: Vec<&DomainRecord> = records
        .iter()
        .filter(|r| r.status == DomainStatus::Accepted)
        .collect();
    for (i, r) in accepted.iter().enumerate() {
        let comma = if i < accepted.len() - 1 { "," } else { "" };
        writeln!(
            file,
            "    {{ \"domain\": {}, \"source\": {}, \"status\": \"accepted\" }}{}",
            json_escape(r.normalized.as_deref().unwrap_or("")),
            json_escape(&r.source),
            comma
        )
        .map_err(|e| format!("Write error: {}", e))?;
    }
    writeln!(file, "  ],").map_err(|e| format!("Write error: {}", e))?;

    // Suspicious records
    writeln!(file, "  \"suspicious\": [").map_err(|e| format!("Write error: {}", e))?;
    let suspicious_list: Vec<&DomainRecord> = records
        .iter()
        .filter(|r| r.status == DomainStatus::Suspicious)
        .collect();
    for (i, r) in suspicious_list.iter().enumerate() {
        let comma = if i < suspicious_list.len() - 1 {
            ","
        } else {
            ""
        };
        writeln!(
            file,
            "    {{ \"domain\": {}, \"source\": {}, \"status\": \"suspicious\", \"reason\": {} }}{}",
            json_escape(r.normalized.as_deref().unwrap_or("")),
            json_escape(&r.source),
            json_escape(&r.reason),
            comma
        )
        .map_err(|e| format!("Write error: {}", e))?;
    }
    writeln!(file, "  ],").map_err(|e| format!("Write error: {}", e))?;

    // Rejected records
    writeln!(file, "  \"rejected\": [").map_err(|e| format!("Write error: {}", e))?;
    let rejected: Vec<&DomainRecord> = records
        .iter()
        .filter(|r| r.status == DomainStatus::Rejected)
        .collect();
    for (i, r) in rejected.iter().enumerate() {
        let comma = if i < rejected.len() - 1 { "," } else { "" };
        writeln!(
            file,
            "    {{ \"original\": {}, \"source\": {}, \"status\": \"rejected\", \"reason\": {} }}{}",
            json_escape(&r.original),
            json_escape(&r.source),
            json_escape(&r.reason),
            comma
        )
        .map_err(|e| format!("Write error: {}", e))?;
    }
    writeln!(file, "  ]").map_err(|e| format!("Write error: {}", e))?;

    writeln!(file, "}}").map_err(|e| format!("Write error: {}", e))?;

    Ok(())
}

/// JSON-escape a string value.
fn json_escape(s: &str) -> String {
    let escaped: String = s
        .chars()
        .flat_map(|c| -> Vec<char> {
            match c {
                '"' => vec!['\\', '"'],
                '\\' => vec!['\\', '\\'],
                '\n' => vec!['\\', 'n'],
                '\r' => vec!['\\', 'r'],
                '\t' => vec!['\\', 't'],
                c if c.is_control() => format!("\\u{:04x}", c as u32).chars().collect(),
                c => vec![c],
            }
        })
        .collect();
    format!("\"{}\"", escaped)
}

fn write_csv(
    path: std::path::PathBuf,
    headers: &[&str],
    records: &[&DomainRecord],
) -> Result<(), String> {
    let mut file = fs::File::create(&path)
        .map_err(|e| format!("Failed to create '{}': {}", path.display(), e))?;

    // Write header
    writeln!(file, "{}", headers.join(","))
        .map_err(|e| format!("Failed to write header to '{}': {}", path.display(), e))?;

    for record in records {
        let normalized = record.normalized.as_deref().unwrap_or("");
        writeln!(
            file,
            "{},{},{},{}",
            escape_csv(normalized),
            escape_csv(&record.source),
            record.status,
            escape_csv(&record.reason),
        )
        .map_err(|e| format!("Failed to write to '{}': {}", path.display(), e))?;
    }

    Ok(())
}

fn write_rejected_csv(path: std::path::PathBuf, records: &[&DomainRecord]) -> Result<(), String> {
    let mut file = fs::File::create(&path)
        .map_err(|e| format!("Failed to create '{}': {}", path.display(), e))?;

    writeln!(file, "original_value,source,status,reason")
        .map_err(|e| format!("Failed to write header to '{}': {}", path.display(), e))?;

    for record in records {
        writeln!(
            file,
            "{},{},{},{}",
            escape_csv(&record.original),
            escape_csv(&record.source),
            record.status,
            escape_csv(&record.reason),
        )
        .map_err(|e| format!("Failed to write to '{}': {}", path.display(), e))?;
    }

    Ok(())
}

fn write_summary(
    path: std::path::PathBuf,
    total: usize,
    valid: usize,
    invalid: usize,
    suspicious: usize,
    allowlisted: usize,
) -> Result<(), String> {
    let mut file = fs::File::create(&path)
        .map_err(|e| format!("Failed to create '{}': {}", path.display(), e))?;

    writeln!(file, "IOCGuard Summary").unwrap();
    writeln!(file, "================").unwrap();
    writeln!(file, "Total lines processed: {}", total).unwrap();
    writeln!(file, "Valid domains:         {}", valid).unwrap();
    writeln!(file, "Invalid domains:       {}", invalid).unwrap();
    writeln!(file, "Suspicious domains:    {}", suspicious).unwrap();
    writeln!(file, "Allowlisted domains:   {}", allowlisted).unwrap();

    Ok(())
}

fn escape_csv(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}
