use std::fs;
use std::process::Command;

use assert_cmd::{cargo::cargo_bin_cmd, prelude::*};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use predicates::prelude::*;
use rusqlite::Connection;
use tempfile::tempdir;

#[test]
fn documented_template_flow_emits_verifiable_receipt() {
    let directory = tempdir().unwrap();
    let database = directory.path().join("pilot.sqlite");
    let connection = Connection::open(&database).unwrap();
    connection
        .execute_batch(
            "CREATE TABLE orders(id INTEGER, account_id TEXT, status TEXT);\
             INSERT INTO orders VALUES (1, 'acct_123', 'open'), (2, 'acct_999', 'closed');",
        )
        .unwrap();
    drop(connection);
    let config = directory.path().join("db-receipts.toml");
    fs::write(
        &config,
        format!(
            "version = 1\nprofile = \"test\"\nreceipt_dir = \"{}\"\ndefault_row_cap = 100\ndefault_column_cap = 12\n\n[[templates]]\nname = \"open-orders\"\ndescription = \"Open orders\"\nsql = \"SELECT id, status FROM orders WHERE account_id = :account_id\"\nparams = [\"account_id\"]\nrow_cap = 50\ncolumn_cap = 6\n",
            directory.path().join("receipts").display()
        ),
    )
    .unwrap();
    let mut command = cargo_bin_cmd!("db-receipts");
    let output = command
        .arg("--config")
        .arg(&config)
        .arg("--json")
        .arg("query")
        .arg("--template")
        .arg("open-orders")
        .arg("--param")
        .arg("account_id=acct_123")
        .env("DB_RECEIPTS_DATABASE_URL", database.to_str().unwrap())
        .env("DB_RECEIPTS_SIGNING_KEY", STANDARD.encode([9_u8; 32]))
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let response: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(response["result"]["row_count"], 1);
    let receipt = response["receipt"].as_str().unwrap();

    let mut verify = Command::new(assert_cmd::cargo::cargo_bin!("db-receipts"));
    verify
        .arg("--json")
        .arg("verify")
        .arg(receipt)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"valid\":true"));

    let receipt_text = fs::read_to_string(receipt).unwrap();
    assert!(!receipt_text.contains("acct_123"));
    assert!(!receipt_text.contains("SELECT id"));
}

#[test]
fn novel_query_is_denied_without_a_terminal_and_receipted() {
    let directory = tempdir().unwrap();
    let config = directory.path().join("db-receipts.toml");
    fs::write(
        &config,
        format!(
            "version = 1\nprofile = \"test\"\nreceipt_dir = \"{}\"\ndefault_row_cap = 10\ndefault_column_cap = 4\n",
            directory.path().join("receipts").display()
        ),
    )
    .unwrap();
    let mut command = cargo_bin_cmd!("db-receipts");
    command
        .arg("--config")
        .arg(&config)
        .arg("query")
        .arg("--sql")
        .arg("SELECT 1")
        .env("DB_RECEIPTS_SIGNING_KEY", STANDARD.encode([9_u8; 32]))
        .assert()
        .code(2)
        .stderr(predicate::str::contains("non-interactive request denied"));
    let receipts = fs::read_dir(directory.path().join("receipts"))
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(receipts.len(), 1);
    assert!(
        fs::read_to_string(receipts[0].path())
            .unwrap()
            .contains("\"outcome\": \"denied\"")
    );
}
