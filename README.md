# dotenv-space CLI

[![CI](https://github.com/urwithajit9/dotenv-space/workflows/CI/badge.svg)](https://github.com/urwithajit9/dotenv-space/actions)
[![Release](https://img.shields.io/github/v/release/urwithajit9/dotenv-space)](https://github.com/urwithajit9/dotenv-space/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A comprehensive CLI tool for managing `.env` files — validation, secret scanning, and format conversion.

**Website:** [dotenv.space](https://dotenv.space)

## Why dotenv-space?

I built this after accidentally pushing AWS credentials to GitHub in a test file during an Airflow refactor (20 DAGs, 300+ Scrapy spiders). The key was revoked immediately, other services went down, and I had to explain the incident to my development head. That conversation was more painful than any billing alert.

Three years later, I'm still paranoid about secrets management. This tool is the safety net I wish I'd had.

## Features

### Phase 1 (v0.1.0) — Available Now

- **`init`** - Interactive project setup, generates `.env.example` for your stack
- **`validate`** - Comprehensive validation with multiple output formats (pretty, JSON, GitHub Actions)
- **`scan`** - Secret detection using pattern matching and entropy analysis
- **`diff`** - Compare `.env` and `.env.example`, find missing/extra variables
- **`convert`** - Transform to 8 different formats (JSON, AWS, GitHub, Docker, Kubernetes, Shell, Terraform, YAML)

### Coming Soon

- **`migrate`** - Direct migration to secret managers (GitHub Actions, Doppler, AWS Secrets Manager)
- **`sync`** - Keep `.env` and `.env.example` in sync
- **`doctor`** - Diagnose common setup issues

## Installation

### Linux / macOS

```bash
curl -sSL https://raw.githubusercontent.com/urwithajit9/dotenv-space/main/install.sh | bash
```

### From source

```bash
cargo install dotenv-space
```

### Verify installation

```bash
dotenv-space --version
```

## Quick Start

```bash
# Create .env.example for your project
dotenv-space init

# Validate your .env file
dotenv-space validate

# Scan for accidentally committed secrets
dotenv-space scan

# Compare files
dotenv-space diff

# Convert to different formats
dotenv-space convert --to json
dotenv-space convert --to aws-secrets
dotenv-space convert --to github-actions
```

## Command Reference

### `dotenv-space init`

Interactive setup for new projects.

```bash
dotenv-space init                                    # Interactive mode
dotenv-space init --stack python --services postgres,redis --yes
```

**Supports:** Python, Node.js, Rust, Go, PHP
**Services:** PostgreSQL, Redis, MongoDB, AWS S3, Stripe, SendGrid, Sentry, OpenAI

### `dotenv-space validate`

Check `.env` against `.env.example` with comprehensive validation.

```bash
dotenv-space validate                                # Pretty output
dotenv-space validate --format json                  # JSON output
dotenv-space validate --format github-actions        # GitHub Actions annotations
dotenv-space validate --strict                       # Fail on warnings
```

**Checks:**
- Missing required variables
- Placeholder values (`YOUR_KEY_HERE`, `CHANGE_ME`, etc.)
- Boolean string trap (`DEBUG="False"` is truthy in Python)
- Weak `SECRET_KEY` (too short, common patterns)
- `localhost` in Docker context
- Extra variables not in `.env.example` (strict mode)

### `dotenv-space scan`

Detect accidentally committed secrets.

```bash
dotenv-space scan                                    # Scan current directory
dotenv-space scan --path src/                        # Scan specific directory
dotenv-space scan --format json                      # JSON output
dotenv-space scan --format sarif                     # SARIF for GitHub Code Scanning
dotenv-space scan --exclude "*.example"              # Exclude patterns
```

**Detects:**
- AWS Access Keys (`AKIA...`)
- Stripe Keys (live and test)
- GitHub Tokens (`ghp_`, `gho_`)
- OpenAI API Keys
- Anthropic API Keys
- Private Keys
- High-entropy strings (possible secrets)

### `dotenv-space diff`

Compare two env files.

```bash
dotenv-space diff                                    # Compare .env and .env.example
dotenv-space diff --show-values                      # Show actual values
dotenv-space diff --format json                      # JSON output
dotenv-space diff --reverse                          # Swap comparison direction
```

### `dotenv-space convert`

Transform `.env` to different formats.

```bash
dotenv-space convert                                 # Interactive mode
dotenv-space convert --to json                       # Generic JSON
dotenv-space convert --to aws-secrets                # AWS Secrets Manager
dotenv-space convert --to github-actions             # GitHub Actions secrets
dotenv-space convert --to docker-compose             # Docker Compose YAML
dotenv-space convert --to kubernetes                 # Kubernetes Secret
dotenv-space convert --to shell                      # Shell export script
dotenv-space convert --to terraform                  # Terraform .tfvars
dotenv-space convert --to yaml                       # Generic YAML

# Advanced options
dotenv-space convert --to json --output secrets.json         # Write to file
dotenv-space convert --to json --include "AWS_*"             # Filter variables
dotenv-space convert --to json --exclude "*_LOCAL"           # Exclude variables
dotenv-space convert --to json --prefix "APP_"               # Add prefix
dotenv-space convert --to json --transform lowercase         # Transform keys
dotenv-space convert --to kubernetes --base64                # Base64-encode values
```

**Pipe to AWS CLI:**
```bash
dotenv-space convert --to aws-secrets | \
  aws secretsmanager create-secret \
    --name prod/myapp/config \
    --secret-string file:///dev/stdin
```

## CI/CD Integration

### GitHub Actions

```yaml
name: Validate Env
on: [push, pull_request]
jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Install dotenv-space
        run: |
          curl -sSL https://raw.githubusercontent.com/urwithajit9/dotenv-space/main/install.sh | bash
      - name: Validate
        run: dotenv-space validate --format github-actions
      - name: Scan for secrets
        run: dotenv-space scan --format sarif
```

### Pre-commit Hook

```yaml
# .pre-commit-config.yaml
repos:
  - repo: local
    hooks:
      - id: dotenv-scan
        name: Scan for secrets
        entry: dotenv-space scan --exit-zero
        language: system
        pass_filenames: false
```

## Configuration

Store preferences in `.dotenv-space.toml`:

```toml
[validate]
strict = true
auto_fix = false

[scan]
exclude_patterns = ["*.example", "*.sample"]
ignore_placeholders = true

[convert]
default_format = "aws-secrets"
```

## Development

```bash
# Clone repo
git clone https://github.com/urwithajit9/dotenv-space.git
cd dotenv-space

# Build
cargo build --release

# Run tests
cargo test

# Run clippy
cargo clippy -- -D warnings

# Format code
cargo fmt
```

## Architecture

- **Parser**: 600 lines, handles all `.env` edge cases (quotes, escapes, expansion)
- **Converter**: Trait-based system, extensible for new formats
- **Scanner**: Pattern matching + entropy analysis for secret detection
- **Commands**: Consistent UX, multiple output formats, proper error handling

See [ARCHITECTURE.md](ARCHITECTURE.md) for details.

## Contributing

Contributions welcome! See [CONTRIBUTING.md](CONTRIBUTING.md).

Areas where help is especially appreciated:
- Additional format converters (Doppler, GCP, Azure)
- Secret pattern improvements
- Documentation improvements
- Integration examples

## Roadmap

**v0.1.0** (Current)
- ✅ init, validate, scan, diff, convert

**v0.2.0** (Q1 2026)
- migrate command (GitHub Actions, Doppler, AWS)
- sync command
- More formats (GCP, Azure, Cloudflare)

**v1.0.0** (Q2 2026)
- doctor command
- backup/restore with encryption
- Template generation
- Homebrew formula

## License

MIT License - see [LICENSE](LICENSE)

## Credits

Built by [Ajit Kumar](https://github.com/urwithajit9) after learning the hard way.

Inspired by countless developers who've accidentally committed secrets. You're not alone.

**Related Projects:**
- [dotenv.space](https://dotenv.space) - Comprehensive .env documentation
- [python-dotenv](https://github.com/theskumar/python-dotenv) - Python implementation
- [dotenvy](https://github.com/allan2/dotenvy) - Rust implementation

## Support

- 🐛 [Report a bug](https://github.com/urwithajit9/dotenv-space/issues/new)
- 💡 [Request a feature](https://github.com/urwithajit9/dotenv-space/issues/new)
- 💬 [Start a discussion](https://github.com/urwithajit9/dotenv-space/discussions)


---

**If this tool saved you from a secrets incident, consider [starring the repo](https://github.com/urwithajit9/dotenv-space) ⭐**