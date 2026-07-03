use std::process;

fn main() {
    let args = match iocguard::cli::parse_args() {
        Ok(args) => args,
        Err(msg) => {
            eprintln!("Error: {}", msg);
            eprintln!("Usage: iocguard validate --input <path> [--allowlist <path>] --out <path> [--json]");
            process::exit(1);
        }
    };

    let result = iocguard::run(
        &args.input,
        args.allowlist.as_deref(),
        &args.out,
        args.json_output,
    );

    match result {
        Ok(()) => {}
        Err(msg) => {
            eprintln!("Error: {}", msg);
            process::exit(1);
        }
    }
}
