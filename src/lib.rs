pub mod cli;
pub mod domain;
pub mod report;

use std::collections::HashSet;
use std::fs;
use std::path::Path;

use domain::{load_allowlist, parse_csv_line, process_line, DomainRecord};

/// Main processing function: reads input, processes domains, generates report.
pub fn run(input_path: &Path, allowlist_path: Option<&Path>, out_dir: &Path) -> Result<(), String> {
    // Read input file
    let input_content = fs::read_to_string(input_path).map_err(|e| {
        format!(
            "Failed to read input file '{}': {}",
            input_path.display(),
            e
        )
    })?;

    // Read allowlist if provided
    let mut allowlist: HashSet<String> = HashSet::new();
    if let Some(path) = allowlist_path {
        let allowlist_content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read allowlist file '{}': {}", path.display(), e))?;
        allowlist = load_allowlist(&allowlist_content);
    }

    // Process each line
    let mut records: Vec<DomainRecord> = Vec::new();
    let mut processed_lines = 0usize;
    let mut header_skipped = false;

    for line in input_content.lines() {
        // Skip header (first line)
        if !header_skipped {
            header_skipped = true;
            continue;
        }

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        processed_lines += 1;

        match parse_csv_line(trimmed) {
            Err(reason) => {
                // Malformed CSV line
                records.push(DomainRecord::rejected(
                    trimmed.to_string(),
                    "unknown".to_string(),
                    format!("Malformed CSV line: {}", reason),
                ));
            }
            Ok((domain, source)) => {
                let record = process_line(domain, source, &allowlist);
                records.push(record);
            }
        }
    }

    // Count statistics
    let total = processed_lines;
    let valid_not_rejected = records
        .iter()
        .filter(|r| r.status != domain::DomainStatus::Rejected)
        .count();
    let invalid_count = records
        .iter()
        .filter(|r| r.status == domain::DomainStatus::Rejected)
        .count();
    let suspicious_count = records
        .iter()
        .filter(|r| r.status == domain::DomainStatus::Suspicious)
        .count();
    let allowlisted_count = records
        .iter()
        .filter(|r| {
            if let Some(ref norm) = r.normalized {
                allowlist.contains(norm) && r.status == domain::DomainStatus::Accepted
            } else {
                false
            }
        })
        .count();

    // Generate report
    crate::report::generate_report(&records, out_dir, allowlisted_count)?;

    // Print terminal summary
    println!("IOCGuard report");
    println!("Processed lines    : {}", total);
    println!("Valid domains      : {}", valid_not_rejected);
    println!("Invalid domains    : {}", invalid_count);
    println!("Allowlisted domains: {}", allowlisted_count);
    println!("Suspicious domains : {}", suspicious_count);
    println!("Report directory   : {}", out_dir.display());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_run_with_sample_data() {
        // This is an integration test
        let input = "data/domains.csv";
        let allowlist = "data/allowlist.txt";
        let out = "test_report";

        // Create the output path relative to workspace
        // Since tests run from the project root
        let out_path = PathBuf::from(out);

        // Remove if exists
        let _ = fs::remove_dir_all(&out_path);

        let result = run(Path::new(input), Some(Path::new(allowlist)), &out_path);
        assert!(result.is_ok());

        // Check files exist
        assert!(out_path.join("accepted.csv").exists());
        assert!(out_path.join("suspicious.csv").exists());
        assert!(out_path.join("rejected.csv").exists());
        assert!(out_path.join("summary.txt").exists());

        // Clean up
        let _ = fs::remove_dir_all(&out_path);
    }
}
