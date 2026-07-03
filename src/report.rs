//! Module for report generation (CSV and summary files).

use std::fs;
use std::io::Write;
use std::path::Path;

use crate::domain::{DomainRecord, DomainStatus};

/// Generate report files in the output directory.
pub fn generate_report(
    records: &[DomainRecord],
    out_dir: &Path,
    allowlisted_count: usize,
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
