//! Module for domain normalization, validation, and suspicion classification.

use std::collections::HashSet;
use std::fmt;

use crate::error::DomainError;

/// Status of a domain after processing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomainStatus {
    Accepted,
    Suspicious,
    Rejected,
}

impl fmt::Display for DomainStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DomainStatus::Accepted => write!(f, "accepted"),
            DomainStatus::Suspicious => write!(f, "suspicious"),
            DomainStatus::Rejected => write!(f, "rejected"),
        }
    }
}

/// A record representing a processed domain entry.
#[derive(Debug, Clone)]
pub struct DomainRecord {
    /// Original raw value from the input (may be malformed).
    pub original: String,
    /// Normalized domain (None if invalid).
    pub normalized: Option<String>,
    /// Source identifier.
    pub source: String,
    /// Processing status.
    pub status: DomainStatus,
    /// Human-readable reason for rejection or suspicion.
    pub reason: String,
}

impl DomainRecord {
    /// Create a new accepted domain record.
    pub fn accepted(original: String, normalized: String, source: String) -> Self {
        DomainRecord {
            original,
            normalized: Some(normalized),
            source,
            status: DomainStatus::Accepted,
            reason: String::new(),
        }
    }

    /// Create a new suspicious domain record.
    pub fn suspicious(
        original: String,
        normalized: String,
        source: String,
        reason: String,
    ) -> Self {
        DomainRecord {
            original,
            normalized: Some(normalized),
            source,
            status: DomainStatus::Suspicious,
            reason,
        }
    }

    /// Create a new rejected domain record with a DomainError.
    pub fn rejected(original: String, source: String, reason: String) -> Self {
        DomainRecord {
            original,
            normalized: None,
            source,
            status: DomainStatus::Rejected,
            reason,
        }
    }
}

/// Normalize a domain string: trim, lowercase, remove trailing dot.
pub fn normalize(domain: &str) -> String {
    let domain = domain.trim();
    let domain = domain.to_lowercase();
    let domain = domain.strip_suffix('.').unwrap_or(&domain);
    domain.to_string()
}

/// Check if a domain is valid after normalization.
/// Returns Ok(()) if valid, or Err(DomainError) if invalid.
pub fn validate(domain: &str) -> Result<(), DomainError> {
    if domain.is_empty() {
        return Err(DomainError::Empty);
    }

    if !domain.contains('.') {
        return Err(DomainError::NoDot);
    }

    if domain.len() > 253 {
        return Err(DomainError::TooLong(domain.len()));
    }

    // Check for consecutive dots
    if domain.contains("..") {
        return Err(DomainError::ConsecutiveDots);
    }

    let labels: Vec<&str> = domain.split('.').collect();

    for label in &labels {
        if label.is_empty() {
            return Err(DomainError::EmptyLabel);
        }

        if label.len() > 63 {
            return Err(DomainError::LabelTooLong(label.len()));
        }

        if label.starts_with('-') || label.ends_with('-') {
            return Err(DomainError::LabelHyphenEdge(label.to_string()));
        }
    }

    // Check characters: only lowercase letters, digits, dots, hyphens allowed
    for c in domain.chars() {
        if !c.is_ascii_lowercase() && !c.is_ascii_digit() && c != '.' && c != '-' {
            return Err(DomainError::InvalidCharacter(c));
        }
    }

    Ok(())
}

/// Suspicion rules for a valid domain.
/// Returns Some(reason) if suspicious, None if clean.
pub fn check_suspicious(domain: &str, allowlist: &HashSet<String>) -> Option<String> {
    // Check allowlist first
    if allowlist.contains(domain) {
        return None;
    }

    let mut reasons: Vec<String> = Vec::new();

    // Rule 1: contains the prefix xn--
    if domain.contains("xn--") {
        reasons.push("contains 'xn--' prefix (internationalized domain)".to_string());
    }

    // Rule 2: top-level domain is one of zip, mov, top, xyz, or tk
    if let Some(tld) = domain.rsplit('.').next() {
        let suspicious_tlds = ["zip", "mov", "top", "xyz", "tk"];
        if suspicious_tlds.contains(&tld) {
            reasons.push(format!("top-level domain '.{}' is suspicious", tld));
        }
    }

    // Rule 3: contains one of the keywords login, verify, secure, update, or paypa1
    let keywords = ["login", "verify", "secure", "update", "paypa1"];
    for kw in &keywords {
        if domain.contains(kw) {
            reasons.push(format!("contains keyword '{}'", kw));
            break;
        }
    }

    // Rule 4: contains three or more hyphens
    let hyphen_count = domain.chars().filter(|&c| c == '-').count();
    if hyphen_count >= 3 {
        reasons.push(format!("contains {} hyphens (>= 3)", hyphen_count));
    }

    if reasons.is_empty() {
        None
    } else {
        Some(reasons.join("; "))
    }
}

/// Process a single CSV line (domain, source) and return a DomainRecord.
pub fn process_line(domain: &str, source: &str, allowlist: &HashSet<String>) -> DomainRecord {
    let original = domain.to_string();
    let source = source.to_string();

    let normalized = normalize(domain);

    match validate(&normalized) {
        Err(e) => DomainRecord::rejected(original, source, e.to_string()),
        Ok(()) => match check_suspicious(&normalized, allowlist) {
            Some(reason) => DomainRecord::suspicious(original, normalized, source, reason),
            None => DomainRecord::accepted(original, normalized, source),
        },
    }
}

/// Parse a CSV line (domain,source). Returns (domain, source) or an error.
pub fn parse_csv_line(line: &str) -> Result<(&str, &str), DomainError> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return Err(DomainError::CsvParseError("Empty line".to_string()));
    }

    // Split on first comma only
    let comma_pos = trimmed
        .find(',')
        .ok_or_else(|| DomainError::CsvParseError("Missing comma separator".to_string()))?;

    let domain = &trimmed[..comma_pos];
    let source = &trimmed[comma_pos + 1..];

    if domain.is_empty() {
        return Err(DomainError::CsvParseError(
            "Domain field is empty".to_string(),
        ));
    }

    Ok((domain, source))
}

/// Load an allowlist file, one domain per line.
pub fn load_allowlist(content: &str) -> HashSet<String> {
    content
        .lines()
        .map(|line| normalize(line.trim()))
        .filter(|d| !d.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::DomainError;

    // === Required tests from specification ===

    #[test]
    fn test_normalize_example_com() {
        assert_eq!(normalize("Example.COM"), "example.com");
    }

    #[test]
    fn test_normalize_trailing_dot() {
        assert_eq!(normalize("updates.example.org."), "updates.example.org");
    }

    #[test]
    fn test_reject_consecutive_dots() {
        let result = validate("bad..domain");
        assert_eq!(result, Err(DomainError::ConsecutiveDots));
    }

    #[test]
    fn test_reject_hyphen_prefix() {
        let result = validate("-malformed.com");
        assert!(result.is_err());
        // The label "-malformed" starts with a hyphen
        assert_eq!(
            result,
            Err(DomainError::LabelHyphenEdge("-malformed".to_string()))
        );
    }

    #[test]
    fn test_suspicious_valid_but_suspicious() {
        let normalized = normalize("paypa1-login.xyz");
        assert_eq!(normalized, "paypa1-login.xyz");
        assert!(validate(&normalized).is_ok());

        let allowlist = HashSet::new();
        let result = check_suspicious(&normalized, &allowlist);
        assert!(result.is_some());
    }

    #[test]
    fn test_allowlisted_not_suspicious() {
        let mut allowlist = HashSet::new();
        allowlist.insert("paypa1-login.xyz".to_string());
        let result = check_suspicious("paypa1-login.xyz", &allowlist);
        assert!(result.is_none());
    }

    // === Additional edge case tests ===

    #[test]
    fn test_empty_domain() {
        assert_eq!(validate(""), Err(DomainError::Empty));
    }

    #[test]
    fn test_no_dot() {
        assert_eq!(validate("localhost"), Err(DomainError::NoDot));
    }

    #[test]
    fn test_long_label() {
        let long = "a".repeat(64) + ".com";
        assert_eq!(validate(&long), Err(DomainError::LabelTooLong(64)));
    }

    #[test]
    fn test_long_domain() {
        let long = "a".repeat(254);
        assert!(validate(&long).is_err());
    }

    #[test]
    fn test_normalize_trim() {
        assert_eq!(normalize("  example.com  "), "example.com");
    }

    #[test]
    fn test_parse_csv_line_valid() {
        let result = parse_csv_line("example.com,manual");
        assert_eq!(result, Ok(("example.com", "manual")));
    }

    #[test]
    fn test_parse_csv_line_missing_comma() {
        assert!(parse_csv_line("example.com").is_err());
    }

    #[test]
    fn test_parse_csv_line_empty_domain() {
        assert!(parse_csv_line(",source").is_err());
    }

    #[test]
    fn test_suspicious_xn() {
        let allowlist = HashSet::new();
        let result = check_suspicious("xn--phishing-test.com", &allowlist);
        assert!(result.is_some());
        assert!(result.unwrap().contains("xn--"));
    }

    #[test]
    fn test_suspicious_tld() {
        let allowlist = HashSet::new();
        let result = check_suspicious("test.xyz", &allowlist);
        assert!(result.is_some());
        assert!(result.unwrap().contains("xyz"));
    }

    #[test]
    fn test_suspicious_keywords() {
        let allowlist = HashSet::new();
        let result = check_suspicious("login-example.com", &allowlist);
        assert!(result.is_some());
        assert!(result.unwrap().contains("login"));
    }

    #[test]
    fn test_suspicious_three_hyphens() {
        let allowlist = HashSet::new();
        let result = check_suspicious("a-b-c-d.com", &allowlist);
        assert!(result.is_some());
        assert!(result.unwrap().contains("hyphens"));
    }

    #[test]
    fn test_process_line_rejected() {
        let allowlist = HashSet::new();
        let record = process_line("bad..domain", "proxy_log", &allowlist);
        assert_eq!(record.status, DomainStatus::Rejected);
        assert!(record.reason.contains("consecutive dots"));
    }

    #[test]
    fn test_process_line_accepted() {
        let mut allowlist = HashSet::new();
        allowlist.insert("example.com".to_string());
        let record = process_line("example.com", "manual", &allowlist);
        assert_eq!(record.status, DomainStatus::Accepted);
    }

    #[test]
    fn test_process_line_suspicious() {
        let allowlist = HashSet::new();
        let record = process_line("paypa1-login.xyz", "email_gateway", &allowlist);
        assert_eq!(record.status, DomainStatus::Suspicious);
    }

    #[test]
    fn test_load_allowlist() {
        let content = "example.com\nsafe-school.edu\n";
        let allowlist = load_allowlist(content);
        assert!(allowlist.contains("example.com"));
        assert!(allowlist.contains("safe-school.edu"));
        assert_eq!(allowlist.len(), 2);
    }

    // === Bonus: Edge case tests ===

    #[test]
    fn test_domain_with_underscore() {
        assert_eq!(
            validate("bad_domain.com"),
            Err(DomainError::InvalidCharacter('_'))
        );
    }

    #[test]
    fn test_domain_with_space() {
        assert_eq!(
            validate("bad domain.com"),
            Err(DomainError::InvalidCharacter(' '))
        );
    }

    #[test]
    fn test_domain_with_uppercase_after_normalize() {
        // After normalization, uppercase becomes lowercase, so this should be valid
        let normalized = normalize("HELLO.COM");
        assert_eq!(normalized, "hello.com");
        assert!(validate(&normalized).is_ok());
    }

    #[test]
    fn test_allowlist_with_trailing_dot() {
        // Allowlist entries are normalized too
        let content = "example.com.\n";
        let allowlist = load_allowlist(content);
        assert!(allowlist.contains("example.com"));
    }

    #[test]
    fn test_multiple_consecutive_dots() {
        assert_eq!(validate("a...b.com"), Err(DomainError::ConsecutiveDots));
    }

    #[test]
    fn test_trailing_dot_valid() {
        // After normalization, trailing dot is removed
        let normalized = normalize("test.example.com.");
        assert_eq!(normalized, "test.example.com");
        assert!(validate(&normalized).is_ok());
    }

    #[test]
    fn test_hyphen_suffix_label() {
        assert_eq!(
            validate("example-.com"),
            Err(DomainError::LabelHyphenEdge("example-".to_string()))
        );
    }

    #[test]
    fn test_all_suspicious_tlds() {
        let allowlist = HashSet::new();
        for tld in &["zip", "mov", "top", "xyz", "tk"] {
            let domain = format!("test.{}", tld);
            let result = check_suspicious(&domain, &allowlist);
            assert!(result.is_some(), "TLD .{} should be suspicious", tld);
            assert!(
                result.unwrap().contains(tld),
                "Reason should mention .{}",
                tld
            );
        }
    }

    #[test]
    fn test_all_suspicious_keywords() {
        let allowlist = HashSet::new();
        for kw in &["login", "verify", "secure", "update", "paypa1"] {
            let domain = format!("{}-test.com", kw);
            let result = check_suspicious(&domain, &allowlist);
            assert!(result.is_some(), "Keyword '{}' should be suspicious", kw);
            assert!(
                result.unwrap().contains(kw),
                "Reason should mention '{}'",
                kw
            );
        }
    }

    #[test]
    fn test_exactly_253_chars() {
        // 4 labels of 63, 63, 63, 62 = 63+1+63+1+63+1+62 = 254... one too many
        // 3 labels of 63 + 1 dot + 63*2 = too many labels
        // Let's do: 63 + '.' + 63 + '.' + 63 + '.' + 61 = 253
        let label = "a".repeat(63);
        let domain = format!("{}.{}.{}.{}", label, label, label, "a".repeat(61));
        assert_eq!(
            domain.len(),
            253,
            "Expected 253 chars, got {}",
            domain.len()
        );
        assert!(validate(&domain).is_ok());
    }

    #[test]
    fn test_exactly_254_chars() {
        // 63 + '.' + 63 + '.' + 63 + '.' + 62 = 254
        let label = "a".repeat(63);
        let domain = format!("{}.{}.{}.{}", label, label, label, "a".repeat(62));
        assert_eq!(
            domain.len(),
            254,
            "Expected 254 chars, got {}",
            domain.len()
        );
        assert!(validate(&domain).is_err());
    }
}
