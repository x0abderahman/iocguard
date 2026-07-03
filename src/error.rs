//! Custom error types for IOCGuard.

use std::fmt;
use std::path::PathBuf;

/// Errors that can occur during CLI argument parsing.
#[derive(Debug, Clone)]
pub enum CliError {
    MissingArgument(String),
    UnknownArgument(String),
    UnknownSubcommand(String),
    Usage,
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CliError::MissingArgument(arg) => write!(f, "Missing required argument --{}", arg),
            CliError::UnknownArgument(arg) => write!(f, "Unknown argument '{}'", arg),
            CliError::UnknownSubcommand(cmd) => {
                write!(f, "Unknown subcommand '{}'. Expected 'validate'", cmd)
            }
            CliError::Usage => write!(
                f,
                "Usage: iocguard validate --input <path> [--allowlist <path>] --out <path> [--json]"
            ),
        }
    }
}

/// Errors that can occur during domain validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomainError {
    Empty,
    NoDot,
    TooLong(usize),
    ConsecutiveDots,
    EmptyLabel,
    LabelTooLong(usize),
    LabelHyphenEdge(String),
    InvalidCharacter(char),
    CsvParseError(String),
}

impl fmt::Display for DomainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DomainError::Empty => write!(f, "Domain is empty"),
            DomainError::NoDot => write!(f, "Domain has no dot"),
            DomainError::TooLong(len) => {
                write!(f, "Domain length {} exceeds maximum of 253", len)
            }
            DomainError::ConsecutiveDots => write!(f, "Domain contains consecutive dots"),
            DomainError::EmptyLabel => write!(f, "Domain contains an empty label"),
            DomainError::LabelTooLong(len) => {
                write!(f, "A label of length {} exceeds maximum of 63", len)
            }
            DomainError::LabelHyphenEdge(label) => {
                write!(f, "Label '{}' starts or ends with a hyphen", label)
            }
            DomainError::InvalidCharacter(c) => {
                write!(f, "Domain contains invalid character '{}'", c)
            }
            DomainError::CsvParseError(msg) => write!(f, "CSV parse error: {}", msg),
        }
    }
}

/// Errors that can occur during report generation.
#[derive(Debug, Clone)]
pub enum ReportError {
    CreateDirectory(PathBuf, String),
    CreateFile(PathBuf, String),
    WriteError(PathBuf, String),
}

impl fmt::Display for ReportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReportError::CreateDirectory(path, msg) => {
                write!(
                    f,
                    "Failed to create directory '{}': {}",
                    path.display(),
                    msg
                )
            }
            ReportError::CreateFile(path, msg) => {
                write!(f, "Failed to create file '{}': {}", path.display(), msg)
            }
            ReportError::WriteError(path, msg) => {
                write!(f, "Failed to write to '{}': {}", path.display(), msg)
            }
        }
    }
}
