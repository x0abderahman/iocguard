# IOCGuard

A small Rust security-oriented command-line utility for defensive security operations.

## Security Use Case

A security analyst receives a list of domain indicators from multiple sources. Some entries are
valid, some are malformed, some are allowlisted, and some look suspicious according to simple
local rules. IOCGuard reads domain indicators, validates and normalizes them, classifies
suspicious ones, and generates a structured report.

## Repository Structure

```
iocguard/
├── Cargo.toml
├── README.md
├── .gitignore
├── data/
│   ├── domains.csv
│   └── allowlist.txt
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── cli.rs
│   ├── domain.rs
│   └── report.rs
└── tests/
    └── integration_tests.rs
```

## Docker Compose Execution Instructions

1. From the directory containing `docker-compose.yml`:
   ```bash
   mkdir -p workspace
   docker compose up -d --build
   docker compose exec rustlab bash
   ```

2. Inside the container:
   ```bash
   cd /workspace/iocguard
   cargo build --release
   ```

## Usage

```bash
cargo run -- validate --input data/domains.csv --allowlist data/allowlist.txt --out report
```

### Arguments

| Argument       | Description                          |
|----------------|--------------------------------------|
| `--input`      | Path to the domain CSV file (required) |
| `--allowlist`  | Path to the allowlist file (optional)  |
| `--out`        | Output report directory (required)     |

## Validation Rules

A domain is rejected if any of these conditions is true:
- It is empty
- It has no dot
- Its total length is greater than 253 characters
- It contains consecutive dots (e.g., `bad..domain`)
- One of its labels is empty
- One of its labels is longer than 63 characters
- One of its labels starts or ends with a hyphen
- It contains characters other than lowercase letters, digits, dot, or hyphen (after normalization)
- The CSV line is malformed

## Suspicion Rules

A valid domain is marked suspicious if it is **not allowlisted** and at least one of these conditions is true:
- It contains the prefix `xn--`
- Its top-level domain is one of: `zip`, `mov`, `top`, `xyz`, `tk`
- It contains one of the keywords: `login`, `verify`, `secure`, `update`, `paypa1`
- It contains three or more hyphens

## Testing and Quality Checks

```bash
# Run unit tests and integration tests
cargo test

# Check formatting
cargo fmt --check

# Run clippy with warnings as errors
cargo clippy -- -D warnings
```

## Output Files

The tool generates the following files in the output directory:

| File             | Description                                |
|------------------|--------------------------------------------|
| `accepted.csv`   | Valid, non-suspicious domains              |
| `suspicious.csv` | Valid but suspicious domains               |
| `rejected.csv`   | Invalid lines with rejection reasons       |
| `summary.txt`    | Human-readable summary of the results      |

## Known Limitations

- The suspicion rules are intentionally simple and not suitable for production threat intelligence.
- The tool uses only the Rust standard library (no external dependencies).
- CSV parsing is basic (no support for quoted fields with commas).
- Only one subcommand (`validate`) is implemented.

## Team Members

- (Student name)
