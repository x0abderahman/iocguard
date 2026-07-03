use std::path::PathBuf;

use crate::error::CliError;

/// Represents the parsed command-line arguments.
pub struct CliArgs {
    pub input: PathBuf,
    pub allowlist: Option<PathBuf>,
    pub out: PathBuf,
    pub json_output: bool,
}

/// Parses command-line arguments manually (no external dependencies).
/// Expected format:
///   iocguard validate --input <path> [--allowlist <path>] --out <path> [--json]
pub fn parse_args() -> Result<CliArgs, CliError> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        return Err(CliError::Usage);
    }

    let subcommand = &args[1];
    if subcommand != "validate" {
        return Err(CliError::UnknownSubcommand(subcommand.clone()));
    }

    let mut input: Option<PathBuf> = None;
    let mut allowlist: Option<PathBuf> = None;
    let mut out: Option<PathBuf> = None;
    let mut json_output = false;

    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--input" => {
                i += 1;
                if i >= args.len() {
                    return Err(CliError::MissingArgument("input".to_string()));
                }
                input = Some(PathBuf::from(&args[i]));
            }
            "--allowlist" => {
                i += 1;
                if i >= args.len() {
                    return Err(CliError::MissingArgument("allowlist".to_string()));
                }
                allowlist = Some(PathBuf::from(&args[i]));
            }
            "--out" => {
                i += 1;
                if i >= args.len() {
                    return Err(CliError::MissingArgument("out".to_string()));
                }
                out = Some(PathBuf::from(&args[i]));
            }
            "--json" => {
                json_output = true;
            }
            _ => {
                return Err(CliError::UnknownArgument(args[i].clone()));
            }
        }
        i += 1;
    }

    let input = input.ok_or_else(|| CliError::MissingArgument("input".to_string()))?;
    let out = out.ok_or_else(|| CliError::MissingArgument("out".to_string()))?;

    Ok(CliArgs {
        input,
        allowlist,
        out,
        json_output,
    })
}
