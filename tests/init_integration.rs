//! Integration tests for `evnx init` command.
#![allow(deprecated)]
mod common;
use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

use common::{read_env_example, count_env_vars};

// ─────────────────────────────────────────────────────────────
// Blank Mode Tests
// ─────────────────────────────────────────────────────────────

#[test]
fn init_blank_creates_minimal_files() {
    let dir = TempDir::new().unwrap();

    // Run: evnx init --yes (defaults to Blueprint, but we'll simulate Blank via stdin)
    // For testing, we use a simpler approach: test the output structure
    Command::cargo_bin("evnx")
        .unwrap()
        .arg("init")
        .arg("--yes")
        .arg("--path")
        .arg(dir.path())
        .write_stdin("0\n")  // Select Blank mode (index 0)
        .assert()
        .success()
        .stdout(predicate::str::contains("Created empty .env.example"));

    // Verify files created
    assert!(dir.path().join(".env.example").exists(), ".env.example should exist");
    assert!(dir.path().join(".env").exists(), ".env should exist");
    assert!(dir.path().join(".gitignore").exists(), ".gitignore should exist");

    // Verify minimal content
    let example = read_env_example(dir.path()).unwrap();
    assert!(example.contains("Add your environment variables here"),
            "Should have placeholder text");
    assert_eq!(count_env_vars(&example), 0, "Should have no actual vars in blank mode");
}

// ─────────────────────────────────────────────────────────────
// Blueprint Mode Tests
// ─────────────────────────────────────────────────────────────

#[test]
fn init_blueprint_t3_modern_generates_expected_vars() {
    let dir = TempDir::new().unwrap();

    Command::cargo_bin("evnx")
        .unwrap()
        .arg("init")
        .arg("--yes")
        .arg("--path")
        .arg(dir.path())
        .write_stdin("1\n0\n")  // Select Blueprint mode, then first blueprint (t3_modern)
        .assert()
        .success()
        .stdout(predicate::str::contains("Created .env.example"));

    let example = read_env_example(dir.path()).unwrap();

    // Check for key vars from T3 blueprint components
    assert!(example.contains("NEXTAUTH_SECRET="), "Should have Next.js auth var");
    assert!(example.contains("CLERK_SECRET_KEY="), "Should have Clerk var");
    assert!(example.contains("DATABASE_URL="), "Should have PostgreSQL var");
    assert!(example.contains("AWS_ACCESS_KEY_ID="), "Should have AWS var");
    assert!(example.contains("STRIPE_SECRET_KEY="), "Should have Stripe var");

    // Check for section organization
    assert!(example.contains("# ── Framework ──"), "Should have Framework section");
    assert!(example.contains("# ── Database ──"), "Should have Database section");
    assert!(example.contains("# ── Payments ──"), "Should have Payments section");

    // Check variable count (T3 has ~15-20 vars)
    let var_count = count_env_vars(&example);
    assert!(var_count >= 15, "Should have at least 15 variables, got {}", var_count);
}

#[test]
fn init_blueprint_rust_high_perf() {
    let dir = TempDir::new().unwrap();

    // Find rust_high_perf in blueprint list and select it
    Command::cargo_bin("evnx")
        .unwrap()
        .arg("init")
        .arg("--yes")
        .arg("--path")
        .arg(dir.path())
        .write_stdin("1\n3\n")  // Blueprint mode, then rust_high_perf (adjust index as needed)
        .assert()
        .success();

    let example = read_env_example(dir.path()).unwrap();

    // Rust-specific vars
    assert!(example.contains("RUST_LOG="), "Should have RUST_LOG");
    assert!(example.contains("SOCKET_ADDR="), "Should have SOCKET_ADDR");

    // Service vars
    assert!(example.contains("DATABASE_URL="), "Should have PostgreSQL");
    assert!(example.contains("REDIS_URL="), "Should have Redis");
    assert!(example.contains("SENTRY_DSN="), "Should have Sentry");
}

// ─────────────────────────────────────────────────────────────
// Architect Mode Tests
// ─────────────────────────────────────────────────────────────

#[test]
fn init_architect_python_django_postgres() {
    let dir = TempDir::new().unwrap();

    // Architect mode: Python → Django → PostgreSQL
    Command::cargo_bin("evnx")
        .unwrap()
        .arg("init")
        .arg("--yes")
        .arg("--path")
        .arg(dir.path())
        .write_stdin("2\n0\n0\n0\n")  // Architect, Python(0), Django(0), PostgreSQL(0), no infra
        .assert()
        .success();

    let example = read_env_example(dir.path()).unwrap();

    // Django framework vars
    assert!(example.contains("SECRET_KEY="), "Should have Django SECRET_KEY");
    assert!(example.contains("DEBUG="), "Should have DEBUG");
    assert!(example.contains("ALLOWED_HOSTS="), "Should have ALLOWED_HOSTS");

    // PostgreSQL service vars
    assert!(example.contains("DATABASE_URL="), "Should have DATABASE_URL");
    assert!(example.contains("DB_HOST="), "Should have DB_HOST");

    // Deduplication: DATABASE_URL should appear once
    let db_url_count = example.lines()
        .filter(|line| line.trim().starts_with("DATABASE_URL="))
        .count();
    assert_eq!(db_url_count, 1, "DATABASE_URL should appear exactly once");
}

#[test]
fn init_architect_multiple_services() {
    let dir = TempDir::new().unwrap();

    // Architect: Rust → Axum → PostgreSQL + Redis + Stripe
    Command::cargo_bin("evnx")
        .unwrap()
        .arg("init")
        .arg("--yes")
        .arg("--path")
        .arg(dir.path())
        .write_stdin("2\n2\n0\n0 1 4\n")  // Architect, Rust, Axum, select postgres(0), redis(1), stripe(4)
        .assert()
        .success();

    let example = read_env_example(dir.path()).unwrap();

    // Should have vars from all three services
    assert!(example.contains("DATABASE_URL="), "Should have PostgreSQL");
    assert!(example.contains("REDIS_URL="), "Should have Redis");
    assert!(example.contains("STRIPE_SECRET_KEY="), "Should have Stripe");

    // Should be organized by category
    assert!(example.contains("# ── Database ──"), "Should have Database section");
    assert!(example.contains("# ── Cache ──"), "Should have Cache section");
    assert!(example.contains("# ── Payments ──"), "Should have Payments section");
}

// ─────────────────────────────────────────────────────────────
// File Operations Tests
// ─────────────────────────────────────────────────────────────

#[test]
fn init_creates_nested_directory() {
    let dir = TempDir::new().unwrap();
    let nested_path = dir.path().join("deep").join("nested").join("project");

    Command::cargo_bin("evnx")
        .unwrap()
        .arg("init")
        .arg("--yes")
        .arg("--path")
        .arg(&nested_path)
        .write_stdin("0\n")  // Blank mode
        .assert()
        .success();

    // Verify directory was created
    assert!(nested_path.exists(), "Nested directory should be created");
    assert!(nested_path.join(".env.example").exists(), ".env.example should exist in nested path");
}

#[test]
fn init_updates_gitignore() {
    let dir = TempDir::new().unwrap();

    // Pre-create .gitignore with some content
    std::fs::write(dir.path().join(".gitignore"), "# My project\n*.log\n").unwrap();

    Command::cargo_bin("evnx")
        .unwrap()
        .arg("init")
        .arg("--yes")
        .arg("--path")
        .arg(dir.path())
        .write_stdin("0\n")  // Blank mode
        .assert()
        .success();

    // Verify .gitignore was updated, not replaced
    let gitignore = std::fs::read_to_string(dir.path().join(".gitignore")).unwrap();
    assert!(gitignore.contains("*.log"), "Original content should be preserved");
    assert!(gitignore.contains(".env\n"), "Should add .env entry");
    assert!(gitignore.contains(".env.local"), "Should add .env.local entry");
}

#[test]
fn init_does_not_overwrite_existing_env() {
    let dir = TempDir::new().unwrap();

    // Pre-create .env with custom content
    let original_env = "MY_CUSTOM_VAR=keep_this\n";
    std::fs::write(dir.path().join(".env"), original_env).unwrap();

    Command::cargo_bin("evnx")
        .unwrap()
        .arg("init")
        .arg("--yes")
        .arg("--path")
        .arg(dir.path())
        .write_stdin("0\n")  // Blank mode
        .assert()
        .success();

    // Verify .env was NOT overwritten
    let env_content = std::fs::read_to_string(dir.path().join(".env")).unwrap();
    assert!(env_content.contains("MY_CUSTOM_VAR=keep_this"),
            "Existing .env should not be overwritten");
}

// ─────────────────────────────────────────────────────────────
// Interactive Mode Simulation Tests
// ─────────────────────────────────────────────────────────────

#[test]
fn init_interactive_mode_selection() {
    let dir = TempDir::new().unwrap();

    // Simulate interactive selection: choose Blueprint, then first blueprint
    Command::cargo_bin("evnx")
        .unwrap()
        .arg("init")
        .arg("--path")
        .arg(dir.path())
        .write_stdin("1\n0\ny\n")  // Blueprint mode, first blueprint, confirm
        .assert()
        .success()
        .stdout(predicate::str::contains("Preview:"))
        .stdout(predicate::str::contains("Generate .env files"));

    assert!(dir.path().join(".env.example").exists());
}

#[test]
fn init_interactive_abort() {
    let dir = TempDir::new().unwrap();

    // Simulate aborting at confirmation
    Command::cargo_bin("evnx")
        .unwrap()
        .arg("init")
        .arg("--path")
        .arg(dir.path())
        .write_stdin("1\n0\nn\n")  // Blueprint, first blueprint, NO to confirm
        .assert()
        .success()
        .stdout(predicate::str::contains("Aborted"));

    // Verify no files were created
    assert!(!dir.path().join(".env.example").exists(),
            "Should not create files when aborted");
}