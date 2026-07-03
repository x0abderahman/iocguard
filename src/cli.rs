use std::path::PathBuf;

/// Represents the parsed command-line arguments.
pub struct CliArgs {
    pub input: PathBuf,
    pub allowlist: Option<PathBuf>,
    pub out: PathBuf,
}

/// Parses command-line arguments manually (no external dependencies).
/// Expected format:
///   iocguard validate --input <path> [--allowlist <path>] --out <path>
pub fn parse_args() -> Result<CliArgs, String> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        return Err(
            "Usage: iocguard validate --input <path> [--allowlist <path>] --out <path>".to_string(),
        );
    }

    let subcommand = &args[1];
    if subcommand != "validate" {
        return Err(format!(
            "Unknown subcommand '{}'. Expected 'validate'.",
            subcommand
        ));
    }

    let mut input: Option<PathBuf> = None;
    let mut allowlist: Option<PathBuf> = None;
    let mut out: Option<PathBuf> = None;

    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--input" => {
                i += 1;
                if i >= args.len() {
                    return Err("Missing value for --input".to_string());
                }
                input = Some(PathBuf::from(&args[i]));
            }
            "--allowlist" => {
                i += 1;
                if i >= args.len() {
                    return Err("Missing value for --allowlist".to_string());
                }
                allowlist = Some(PathBuf::from(&args[i]));
            }
            "--out" => {
                i += 1;
                if i >= args.len() {
                    return Err("Missing value for --out".to_string());
                }
                out = Some(PathBuf::from(&args[i]));
            }
            _ => {
                return Err(format!("Unknown argument '{}'", args[i]));
            }
        }
        i += 1;
    }

    let input = input.ok_or_else(|| "Missing required argument --input".to_string())?;
    let out = out.ok_or_else(|| "Missing required argument --out".to_string())?;

    Ok(CliArgs {
        input,
        allowlist,
        out,
    })
}

#[cfg(test)]
mod tests {
    // parse_args reads from env::args(), so unit testing it directly is not practical.
    // Integration tests in tests/ cover the full pipeline.
}
