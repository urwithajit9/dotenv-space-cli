## dotenv-space CLI

**A powerful Rust CLI for managing environment variables — validation, scanning, conversion, and migration.**

dotenv-space solves the real-world problems developers face daily when working with `.env` files and secret management systems. It helps you validate configs, detect leaked secrets, convert formats, and migrate to modern secret managers — all locally, securely, and with zero telemetry.

---

## ✨ Features

* Interactive project initialization
* Validate `.env` against `.env.example`
* Detect accidentally committed secrets
* Convert `.env` to multiple formats (JSON, YAML, Terraform, Docker, etc.)
* Migration workflows to secret managers
* Sync `.env` and `.env.example`
* Template rendering from env variables
* Encrypted backups
* Project diagnostics
* Beautiful colored terminal output
* Fully scriptable for CI/CD

---

## 🧠 Design Principles

* Interactive by default, scriptable via flags
* Fail fast with clear errors
* Zero configuration for common cases
* Idempotent operations
* Respect `.gitignore`
* No network calls unless explicitly migrating

---

## 📦 Installation

### Using Cargo (recommended)

```bash
cargo install dotenv-space
```

### From source

```bash
git clone https://github.com/your-org/dotenv-space
cd dotenv-space
cargo build --release
```

Binary will be located at:

```
target/release/dotenv-space
```

---

## 🚀 Quick Start

### Initialize a project

```bash
dotenv-space init
```

Creates:

* `.env.example`
* `.env`
* Updates `.gitignore`

---

### Validate environment variables

```bash
dotenv-space validate
```

Checks for:

* Missing variables
* Placeholder values
* Weak secrets
* Misconfigurations

---

### Scan for exposed secrets

```bash
dotenv-space scan
```

Detects patterns like:

* AWS keys
* Stripe keys
* GitHub tokens
* High-entropy secrets

---

### Compare env files

```bash
dotenv-space diff
```

Shows missing, extra, and changed variables.

---

### Convert formats

```bash
dotenv-space convert --to json
```

Supports:

* AWS Secrets Manager JSON
* GitHub Actions format
* Docker Compose YAML
* Kubernetes secrets
* Terraform tfvars
* Generic JSON/YAML
* Shell export script

---

## 🧰 Command Reference

```
dotenv-space <COMMAND> [OPTIONS]

Commands:
  init       Interactive project setup
  validate   Validate .env against .env.example
  scan       Detect exposed secrets
  diff       Compare .env and .env.example
  convert    Convert to different formats
  migrate    Migration workflows
  sync       Sync env files
  template   Render templates
  backup     Create encrypted backup
  restore    Restore backup
  doctor     Diagnose issues
```

---

## 🏗 Architecture

```
src/
 ├── main.rs
 ├── commands/
 ├── core/
 ├── formats/
 ├── templates/
 └── utils/
```

### Modules

* **commands** → CLI command handlers
* **core** → parser, validator, scanner
* **formats** → output converters
* **templates** → init templates
* **utils** → helpers (git, UI, patterns)

---

## 🔐 Security Model

* Client-side only
* No telemetry
* No background services
* Secrets never transmitted unless explicitly migrating
* Optional encrypted backups using AES-256-GCM

---

## 🧪 Testing

```bash
cargo test
```

Coverage:

```bash
cargo tarpaulin --out Html
```

---

## ⚙️ Configuration

Supports optional config file:

```
.dotenv-space.toml
```

Example:

```toml
[project]
name = "my-app"
stack = "python"

[validate]
strict = true
```

---

## 🧩 Shell Completions

```bash
dotenv-space completions bash > ~/.local/share/bash-completion/completions/dotenv-space
```

Supports:

* Bash
* Zsh
* Fish
* PowerShell

---

## 📊 Roadmap

### Phase 1 (MVP)

* Init
* Validate
* Scan
* Diff

### Phase 2

* Convert
* Migrate
* Sync

### Phase 3

* Templates
* Backup / Restore
* Doctor

---

## 🚫 Non-Goals

* Secret storage service
* Daemon/service mode
* Web UI
* Runtime env injection
* Database integrations

---

## 🤔 Why Rust?

* Single static binary
* High performance
* Strong type safety
* Excellent CLI ecosystem
* No runtime dependencies

---

## 🤝 Contributing

Contributions are welcome.

Steps:

1. Fork repository
2. Create feature branch
3. Add tests
4. Submit PR

---

## 📄 License

MIT License (recommended — adjust if needed)

---

## ⭐ Vision

dotenv-space aims to become the standard CLI for environment variable management across stacks — from local development to production secret migrations — with a focus on safety, usability, and developer experience.


