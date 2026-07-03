// Integration tests for the full pipeline.
// These are separate from the unit tests embedded in each module.

use std::fs;
use std::path::Path;

/// Test the full pipeline with sample data.
#[test]
fn test_full_pipeline() {
    let input = Path::new("data/domains.csv");
    let allowlist = Path::new("data/allowlist.txt");
    let out_dir = Path::new("test_report_full");

    // Clean up any previous test artifacts
    let _ = fs::remove_dir_all(out_dir);

    let result = iocguard::run(input, Some(allowlist), out_dir, false);
    assert!(result.is_ok(), "Pipeline run failed: {:?}", result);

    // Check output files
    assert!(
        out_dir.join("accepted.csv").exists(),
        "accepted.csv not found"
    );
    assert!(
        out_dir.join("suspicious.csv").exists(),
        "suspicious.csv not found"
    );
    assert!(
        out_dir.join("rejected.csv").exists(),
        "rejected.csv not found"
    );
    assert!(
        out_dir.join("summary.txt").exists(),
        "summary.txt not found"
    );

    // Verify summary content
    let summary = fs::read_to_string(out_dir.join("summary.txt")).unwrap();
    assert!(summary.contains("Total lines processed: 9"));
    assert!(summary.contains("Valid domains:"));
    assert!(summary.contains("Invalid domains:"));

    // Verify accepted.csv has expected content
    let accepted = fs::read_to_string(out_dir.join("accepted.csv")).unwrap();
    assert!(accepted.contains("example.com,manual"));
    assert!(accepted.contains("safe-school.edu,manual"));

    // Verify suspicious.csv
    let suspicious = fs::read_to_string(out_dir.join("suspicious.csv")).unwrap();
    assert!(suspicious.contains("paypa1-login.xyz"));
    assert!(suspicious.contains("xn--phishing-test.com"));

    // Verify rejected.csv
    let rejected = fs::read_to_string(out_dir.join("rejected.csv")).unwrap();
    assert!(rejected.contains("bad..domain"));
    assert!(rejected.contains("-malformed.com"));

    // Clean up
    let _ = fs::remove_dir_all(out_dir);
}

/// Test without allowlist.
#[test]
fn test_without_allowlist() {
    let input = Path::new("data/domains.csv");
    let out_dir = Path::new("test_report_no_allowlist");

    let _ = fs::remove_dir_all(out_dir);

    let result = iocguard::run(input, None, out_dir, false);
    assert!(result.is_ok());

    assert!(out_dir.join("accepted.csv").exists());
    assert!(out_dir.join("suspicious.csv").exists());

    let _ = fs::remove_dir_all(out_dir);
}

/// Test JSON output format.
#[test]
fn test_json_output() {
    let input = Path::new("data/domains.csv");
    let allowlist = Path::new("data/allowlist.txt");
    let out_dir = Path::new("test_report_json");

    let _ = fs::remove_dir_all(out_dir);

    let result = iocguard::run(input, Some(allowlist), out_dir, true);
    assert!(result.is_ok(), "JSON pipeline failed: {:?}", result);

    // Check JSON file exists
    assert!(
        out_dir.join("report.json").exists(),
        "report.json not found"
    );

    // Verify JSON content
    let json_content = fs::read_to_string(out_dir.join("report.json")).unwrap();
    assert!(json_content.contains("\"summary\""));
    assert!(json_content.contains("\"accepted\""));
    assert!(json_content.contains("\"suspicious\""));
    assert!(json_content.contains("\"rejected\""));
    assert!(json_content.contains("example.com"));
    assert!(json_content.contains("paypa1-login.xyz"));
    assert!(json_content.contains("bad..domain"));

    // Verify summary.txt still exists
    assert!(
        out_dir.join("summary.txt").exists(),
        "summary.txt not found"
    );

    let _ = fs::remove_dir_all(out_dir);
}
