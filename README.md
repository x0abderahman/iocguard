# IOCGuard

A small Rust security-oriented command-line utility for defensive security operations.

## Security Use Case

A security analyst receives a list of domain indicators from multiple sources. Some entries are
valid, some are malformed, some are allowlisted, and some look suspicious according to simple
local rules. IOCGuard reads domain indicators, validates and normalizes them, classifies
suspicious ones, and generates a structured report (CSV or JSON).

## Repository Structure

```
iocguard/
├── Cargo.toml
├── README.md
├── .gitignore
├── data/
│   ├── domains.csv
│   └── allowlist.txt
├── screenshots/
│   ├── 01_docker_environment.png
│   ├── 02_cargo_run.png
│   ├── 03_cargo_test.png
│   ├── 04_cargo_fmt_clippy.png
│   ├── 05_report_files.png
│   ├── 06_cargo_run_json.png
│   ├── 07_report_files_json.png
│   └── 08_cargo_test_json.png
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── cli.rs
│   ├── domain.rs
│   ├── error.rs         # Custom error types (bonus)
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

### Standard CSV output
```bash
cargo run -- validate --input data/domains.csv --allowlist data/allowlist.txt --out report
```

### JSON output (bonus feature)
```bash
cargo run -- validate --input data/domains.csv --allowlist data/allowlist.txt --out report --json
```

### Arguments

| Argument       | Description                                   |
|----------------|-----------------------------------------------|
| `--input`      | Path to the domain CSV file (required)        |
| `--allowlist`  | Path to the allowlist file (optional)         |
| `--out`        | Output report directory (required)            |
| `--json`       | Generate JSON output instead of CSV (bonus)   |

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

## Bonus Features

### 1. JSON Output (`--json` flag)
When `--json` is provided, the tool generates `report.json` with structured data instead of CSV files.
The JSON includes summary statistics, accepted domains, suspicious domains with reasons, and
rejected entries with original values.

### 2. Custom Error Types
The project uses three custom error enums instead of generic strings:
- `CliError` - for CLI argument parsing errors
- `DomainError` - for domain validation errors (Empty, NoDot, ConsecutiveDots, etc.)
- `ReportError` - for file/report generation errors

### 3. Extended Test Suite
38 tests in total covering:
- Required specification tests (normalization, validation, suspicion, allowlist)
- Edge cases: underscores, spaces, uppercase, multiple consecutive dots
- Boundary tests: exactly 253/254 character domains
- All suspicious TLDs and keywords tested individually

## Testing and Quality Checks

```bash
# Run all 38 tests
cargo test

# Check formatting
cargo fmt --check

# Run clippy with warnings as errors
cargo clippy -- -D warnings
```

## Output Files

### CSV mode (default)
| File             | Description                                |
|------------------|--------------------------------------------|
| `accepted.csv`   | Valid, non-suspicious domains              |
| `suspicious.csv` | Valid but suspicious domains               |
| `rejected.csv`   | Invalid lines with rejection reasons       |
| `summary.txt`    | Human-readable summary of the results      |

### JSON mode (`--json`)
| File             | Description                                |
|------------------|--------------------------------------------|
| `report.json`    | Structured JSON with all results           |
| `summary.txt`    | Human-readable summary of the results      |

## Known Limitations

- The suspicion rules are intentionally simple and not suitable for production threat intelligence.
- The tool uses only the Rust standard library (no external dependencies).
- CSV parsing is basic (no support for quoted fields with commas).
- Only one subcommand (`validate`) is implemented.

## Team Members

- Abderahman Mohamed Lemin (ID: 25235)
