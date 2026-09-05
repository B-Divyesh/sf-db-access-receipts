use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};

use assert_cmd::{cargo::cargo_bin, cargo::cargo_bin_cmd, prelude::*};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use db_access_receipts::{SignedReceipt, execute_readonly, verify_receipt};
use predicates::prelude::*;
use rusqlite::Connection;
use serde_json::Value;
use tempfile::{TempDir, tempdir};
use uuid::Uuid;

fn signing_key() -> String {
    STANDARD.encode([17_u8; 32])
}

fn create_orders(directory: &TempDir, rows: usize) -> std::path::PathBuf {
    let database = directory.path().join("orders.sqlite");
    let connection = Connection::open(&database).unwrap();
    connection
        .execute_batch(
            "CREATE TABLE orders(id INTEGER, account_id TEXT, status TEXT, created_at TEXT);",
        )
        .unwrap();
    for id in 0..rows {
        connection
            .execute(
                "INSERT INTO orders VALUES (?1, ?2, ?3, ?4)",
                (
                    id as i64,
                    "acct_demo",
                    "open",
                    format!("2026-08-{id:02}T09:14:00Z"),
                ),
            )
            .unwrap();
    }
    database
}

fn config_with_templates(
    directory: &TempDir,
    templates: &str,
    row_cap: usize,
    column_cap: usize,
) -> std::path::PathBuf {
    let config = directory.path().join("db-receipts.toml");
    fs::write(
        &config,
        format!(
            "version = 1\nprofile = \"claim-{}\"\nreceipt_dir = \"{}\"\ndefault_row_cap = {row_cap}\ndefault_column_cap = {column_cap}\n{templates}",
            Uuid::new_v4(),
            directory.path().join("receipts").display(),
        ),
    )
    .unwrap();
    config
}

fn template_config(directory: &TempDir) -> std::path::PathBuf {
    config_with_templates(
        directory,
        r#"
[[templates]]
name = "open-orders"
description = "Open demo orders"
sql = "SELECT id, status, created_at FROM orders WHERE account_id = :account_id"
params = ["account_id"]
row_cap = 50
column_cap = 6
"#,
        100,
        12,
    )
}

fn receipts(directory: &TempDir) -> Vec<(std::path::PathBuf, SignedReceipt)> {
    fs::read_dir(directory.path().join("receipts"))
        .unwrap()
        .map(|entry| {
            let path = entry.unwrap().path();
            let receipt = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
            (path, receipt)
        })
        .collect()
}

#[test]
#[doc = "@claim:cli-demo"]
fn claim_cli_demo_creates_sample_receipt_that_verifies() {
    let output = cargo_bin_cmd!("db-receipts")
        .arg("--json")
        .arg("demo")
        .output()
        .unwrap();
    assert!(output.status.success());
    let response: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(response["demo"], true);
    assert_eq!(response["result"]["row_count"], 2);
    let receipt = response["receipt"].as_str().unwrap();
    Command::new(cargo_bin!("db-receipts"))
        .arg("--json")
        .arg("verify")
        .arg(receipt)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"valid\":true"));
    fs::remove_dir_all(response["directory"].as_str().unwrap()).unwrap();
}

#[test]
#[doc = "@claim:cli-no-telemetry"]
fn claim_cli_demo_runs_with_network_sockets_blocked() {
    let directory = tempdir().unwrap();
    let source = directory.path().join("block_network.c");
    let library = directory.path().join("block_network.so");
    fs::write(
        &source,
        r#"#include <errno.h>
int socket(int domain, int type, int protocol) { errno = EPERM; return -1; }
int connect(int socket, const void *address, unsigned int length) { errno = EPERM; return -1; }
"#,
    )
    .unwrap();
    Command::new("cc")
        .args(["-shared", "-fPIC", "-o"])
        .arg(&library)
        .arg(&source)
        .assert()
        .success();
    Command::new(cargo_bin!("db-receipts"))
        .arg("--json")
        .arg("demo")
        .env("LD_PRELOAD", &library)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"demo\":true"));
}

#[test]
#[doc = "@claim:templates-and-limits"]
fn claim_templates_and_limits_require_matching_parameters_and_cap_rows() {
    let directory = tempdir().unwrap();
    let database = create_orders(&directory, 60);
    let config = template_config(&directory);
    let output = cargo_bin_cmd!("db-receipts")
        .arg("--config")
        .arg(&config)
        .arg("--json")
        .arg("query")
        .arg("--template")
        .arg("open-orders")
        .arg("--param")
        .arg("account_id=acct_demo")
        .env("DB_RECEIPTS_DATABASE_URL", &database)
        .env("DB_RECEIPTS_SIGNING_KEY", signing_key())
        .output()
        .unwrap();
    assert!(output.status.success());
    let response: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(response["result"]["row_count"], 50);
    assert_eq!(response["result"]["truncated"], true);

    cargo_bin_cmd!("db-receipts")
        .arg("--config")
        .arg(&config)
        .arg("query")
        .arg("--template")
        .arg("open-orders")
        .env("DB_RECEIPTS_DATABASE_URL", &database)
        .env("DB_RECEIPTS_SIGNING_KEY", signing_key())
        .assert()
        .code(2)
        .stderr(predicate::str::contains("query parameters do not match"));
    assert!(
        receipts(&directory)
            .iter()
            .any(|(_, receipt)| receipt.payload.outcome == "denied")
    );
}

#[test]
#[doc = "@claim:noninteractive-novel-denial"]
fn claim_noninteractive_novel_query_is_denied_and_receipted() {
    let directory = tempdir().unwrap();
    let config = config_with_templates(&directory, "", 10, 4);
    cargo_bin_cmd!("db-receipts")
        .arg("--config")
        .arg(&config)
        .arg("query")
        .arg("--sql")
        .arg("SELECT 1")
        .env("DB_RECEIPTS_SIGNING_KEY", signing_key())
        .assert()
        .code(2)
        .stderr(predicate::str::contains("non-interactive request denied"));
    let receipt = receipts(&directory).pop().unwrap().1;
    assert_eq!(receipt.payload.outcome, "denied");
    assert_eq!(receipt.payload.approval, "not-approved");
    verify_receipt(&receipt).unwrap();
}

#[test]
#[doc = "@claim:novel-human-challenge"]
fn claim_novel_query_in_a_terminal_requires_a_one_use_human_challenge() {
    let directory = tempdir().unwrap();
    let config = config_with_templates(&directory, "", 10, 4);
    let command = format!(
        "{} --config '{}' query --sql 'SELECT 1'",
        cargo_bin!("db-receipts").display(),
        config.display(),
    );
    let mut child = Command::new("script")
        .args(["-qefc", &command, "/dev/null"])
        .env("DB_RECEIPTS_SIGNING_KEY", signing_key())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(b"wrong-code\n")
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert_eq!(output.status.code(), Some(2));
    let terminal = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(terminal.contains("Novel query approval required"));
    assert!(terminal.contains("Type "));
    let receipt = receipts(&directory).pop().unwrap().1;
    assert_eq!(receipt.payload.outcome, "denied");
    assert_eq!(receipt.payload.approval, "not-approved");
}

#[test]
#[doc = "@claim:readonly-write-denial"]
fn claim_readonly_connection_refuses_writes_without_changing_data() {
    let directory = tempdir().unwrap();
    let database = create_orders(&directory, 3);
    let result = execute_readonly(
        database.to_str().unwrap(),
        "DELETE FROM orders",
        &Default::default(),
        None,
        10,
        4,
    );
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("write or schema-changing SQL")
    );
    let connection = Connection::open(&database).unwrap();
    let remaining: i64 = connection
        .query_row("SELECT count(*) FROM orders", [], |row| row.get(0))
        .unwrap();
    assert_eq!(remaining, 3);
}

#[test]
#[doc = "@claim:column-cap"]
fn claim_column_caps_reject_over_broad_queries() {
    let directory = tempdir().unwrap();
    let database = create_orders(&directory, 1);
    let result = execute_readonly(
        database.to_str().unwrap(),
        "SELECT id, account_id, status FROM orders",
        &Default::default(),
        None,
        10,
        2,
    );
    assert!(result.unwrap_err().to_string().contains("policy cap is 2"));
}

#[test]
#[doc = "@claim:signed-attempts"]
fn claim_success_denial_and_query_failure_are_all_signed() {
    let directory = tempdir().unwrap();
    let database = create_orders(&directory, 1);
    let config = config_with_templates(
        &directory,
        r#"
[[templates]]
name = "good"
description = "Good query"
sql = "SELECT id FROM orders WHERE account_id = :account_id"
params = ["account_id"]

[[templates]]
name = "broken"
description = "Broken query"
sql = "SELECT id FROM missing_table"
params = []
"#,
        10,
        4,
    );
    let binary = cargo_bin!("db-receipts");
    for args in [
        vec![
            "query",
            "--template",
            "good",
            "--param",
            "account_id=acct_demo",
        ],
        vec!["query", "--template", "good"],
        vec!["query", "--template", "broken"],
    ] {
        let status = Command::new(&binary)
            .arg("--config")
            .arg(&config)
            .args(args)
            .env("DB_RECEIPTS_DATABASE_URL", &database)
            .env("DB_RECEIPTS_SIGNING_KEY", signing_key())
            .status()
            .unwrap();
        assert!(status.code().is_some());
    }
    let all = receipts(&directory);
    assert!(
        all.iter()
            .any(|(_, receipt)| receipt.payload.outcome == "allowed")
    );
    assert!(
        all.iter()
            .any(|(_, receipt)| receipt.payload.outcome == "denied")
    );
    assert!(
        all.iter()
            .any(|(_, receipt)| receipt.payload.outcome == "failed")
    );
    for (_, receipt) in all {
        verify_receipt(&receipt).unwrap();
    }
}

#[test]
#[doc = "@claim:offline-verification"]
fn claim_receipts_verify_offline_and_tampering_fails() {
    let directory = tempdir().unwrap();
    let database = create_orders(&directory, 1);
    let config = template_config(&directory);
    let output = cargo_bin_cmd!("db-receipts")
        .arg("--config")
        .arg(&config)
        .arg("--json")
        .arg("query")
        .arg("--template")
        .arg("open-orders")
        .arg("--param")
        .arg("account_id=acct_demo")
        .env("DB_RECEIPTS_DATABASE_URL", &database)
        .env("DB_RECEIPTS_SIGNING_KEY", signing_key())
        .output()
        .unwrap();
    assert!(output.status.success());
    let receipt = serde_json::from_slice::<Value>(&output.stdout).unwrap()["receipt"]
        .as_str()
        .unwrap()
        .to_owned();
    Command::new(cargo_bin!("db-receipts"))
        .arg("verify")
        .arg(&receipt)
        .assert()
        .success();
    let tampered = directory.path().join("tampered.json");
    let mut changed: Value = serde_json::from_slice(&fs::read(&receipt).unwrap()).unwrap();
    changed["payload"]["actor"] = Value::String("tampered@example.test".into());
    fs::write(&tampered, serde_json::to_vec_pretty(&changed).unwrap()).unwrap();
    Command::new(cargo_bin!("db-receipts"))
        .arg("verify")
        .arg(&tampered)
        .assert()
        .code(4)
        .stderr(predicate::str::contains("signature is invalid"));
}

#[test]
#[doc = "@claim:receipt-minimization"]
fn claim_receipts_omit_values_sql_paths_and_result_cells() {
    let directory = tempdir().unwrap();
    let database = create_orders(&directory, 1);
    let config = template_config(&directory);
    let output = cargo_bin_cmd!("db-receipts")
        .arg("--config")
        .arg(&config)
        .arg("--json")
        .arg("query")
        .arg("--template")
        .arg("open-orders")
        .arg("--param")
        .arg("account_id=acct_demo")
        .env("DB_RECEIPTS_DATABASE_URL", &database)
        .env("DB_RECEIPTS_SIGNING_KEY", signing_key())
        .output()
        .unwrap();
    assert!(output.status.success());
    let receipt = serde_json::from_slice::<Value>(&output.stdout).unwrap()["receipt"]
        .as_str()
        .unwrap()
        .to_owned();
    let stored = fs::read_to_string(receipt).unwrap();
    for forbidden in [
        "acct_demo",
        "SELECT id",
        database.to_str().unwrap(),
        "2026-08-00T09:14:00Z",
    ] {
        assert!(!stored.contains(forbidden), "receipt exposed {forbidden}");
    }
}

#[test]
#[doc = "@claim:json-exit-codes"]
fn claim_json_output_and_exit_codes_cover_success_and_denial() {
    let directory = tempdir().unwrap();
    let database = create_orders(&directory, 1);
    let config = template_config(&directory);
    let success = cargo_bin_cmd!("db-receipts")
        .arg("--config")
        .arg(&config)
        .arg("--json")
        .arg("query")
        .arg("--template")
        .arg("open-orders")
        .arg("--param")
        .arg("account_id=acct_demo")
        .env("DB_RECEIPTS_DATABASE_URL", &database)
        .env("DB_RECEIPTS_SIGNING_KEY", signing_key())
        .output()
        .unwrap();
    assert!(success.status.success());
    assert_eq!(
        serde_json::from_slice::<Value>(&success.stdout).unwrap()["ok"],
        true
    );
    let denial = cargo_bin_cmd!("db-receipts")
        .arg("--config")
        .arg(&config)
        .arg("--json")
        .arg("query")
        .arg("--template")
        .arg("unknown")
        .env("DB_RECEIPTS_DATABASE_URL", &database)
        .env("DB_RECEIPTS_SIGNING_KEY", signing_key())
        .output()
        .unwrap();
    assert_eq!(denial.status.code(), Some(2));
    assert_eq!(
        serde_json::from_slice::<Value>(&denial.stdout).unwrap()["exit_code"],
        2
    );
}

#[test]
#[doc = "@claim:no-plaintext-secret-fallback"]
fn claim_keychain_failure_never_creates_a_plaintext_secret_file() {
    let directory = tempdir().unwrap();
    let config = config_with_templates(&directory, "", 10, 4);
    let database_url = format!("sqlite:///tmp/claim-secret-{}.sqlite", Uuid::new_v4());
    let output = cargo_bin_cmd!("db-receipts")
        .arg("--config")
        .arg(&config)
        .arg("secret")
        .arg("set")
        .arg("--database-url")
        .arg(&database_url)
        .output()
        .unwrap();
    let profile = fs::read_to_string(&config).unwrap();
    assert!(!profile.contains(&database_url));
    let files = fs::read_dir(directory.path())
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(
        files.len(),
        1,
        "secret set must not create a plaintext fallback file"
    );
    if output.status.success() {
        let _ = cargo_bin_cmd!("db-receipts")
            .arg("--config")
            .arg(&config)
            .arg("secret")
            .arg("clear")
            .output();
    }
}
