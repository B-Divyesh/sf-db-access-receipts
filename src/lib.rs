use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use base64::{Engine as _, engine::general_purpose::STANDARD};
use chrono::{DateTime, SecondsFormat, Utc};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::{RngCore, rngs::OsRng};
use rusqlite::{Connection, OpenFlags, types::ValueRef};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;
use uuid::Uuid;

pub const KEYRING_SERVICE: &str = "db-access-receipts";

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    Policy(String),
    #[error("{0}")]
    Database(String),
    #[error("{0}")]
    Receipt(String),
    #[error("{0}")]
    Input(String),
}

impl Error {
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::Policy(_) | Self::Input(_) => 2,
            Self::Database(_) => 3,
            Self::Receipt(_) => 4,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub version: u8,
    #[serde(default = "default_profile")]
    pub profile: String,
    pub receipt_dir: PathBuf,
    pub default_row_cap: usize,
    pub default_column_cap: usize,
    #[serde(default)]
    pub templates: Vec<QueryTemplate>,
}

fn default_profile() -> String {
    "default".to_owned()
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct QueryTemplate {
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub sql: String,
    #[serde(default)]
    pub params: Vec<String>,
    #[serde(default)]
    pub row_cap: Option<usize>,
    #[serde(default)]
    pub column_cap: Option<usize>,
}

impl Config {
    pub fn load(path: &Path) -> Result<Self, Error> {
        let source = fs::read_to_string(path)
            .map_err(|e| Error::Input(format!("could not read policy {}: {e}", path.display())))?;
        let config: Self = toml::from_str(&source)
            .map_err(|e| Error::Input(format!("invalid policy {}: {e}", path.display())))?;
        config.validate()?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<(), Error> {
        if self.version != 1 {
            return Err(Error::Input(format!(
                "unsupported policy version {}; expected 1",
                self.version
            )));
        }
        if self.profile.trim().is_empty() {
            return Err(Error::Input("policy profile cannot be empty".into()));
        }
        if self.default_row_cap == 0 || self.default_column_cap == 0 {
            return Err(Error::Input(
                "row and column caps must be greater than zero".into(),
            ));
        }
        let mut names = BTreeSet::new();
        for template in &self.templates {
            if template.name.trim().is_empty() {
                return Err(Error::Input("template name cannot be empty".into()));
            }
            if !names.insert(template.name.as_str()) {
                return Err(Error::Input(format!(
                    "duplicate template name: {}",
                    template.name
                )));
            }
            if template.row_cap == Some(0) || template.column_cap == Some(0) {
                return Err(Error::Input(format!(
                    "template {} has a zero cap",
                    template.name
                )));
            }
            validate_param_names(&template.params)?;
        }
        Ok(())
    }

    pub fn template(&self, name: &str) -> Result<&QueryTemplate, Error> {
        self.templates
            .iter()
            .find(|template| template.name == name)
            .ok_or_else(|| Error::Policy(format!("template not allowlisted: {name}")))
    }

    pub fn caps_for(&self, template: Option<&QueryTemplate>) -> (usize, usize) {
        match template {
            Some(template) => (
                template.row_cap.unwrap_or(self.default_row_cap),
                template.column_cap.unwrap_or(self.default_column_cap),
            ),
            None => (self.default_row_cap, self.default_column_cap),
        }
    }
}

pub fn initial_config(profile: &str) -> String {
    format!(
        r#"version = 1
profile = "{profile}"
receipt_dir = ".db-receipts/receipts"
default_row_cap = 100
default_column_cap = 12

# Add reviewed, named query templates here. Values are always bound parameters.
# [[templates]]
# name = "open-orders"
# description = "Open orders for one account"
# sql = "SELECT id, status FROM orders WHERE account_id = :account_id"
# params = ["account_id"]
# row_cap = 50
# column_cap = 6
"#
    )
}

fn validate_param_names(names: &[String]) -> Result<(), Error> {
    let mut unique = BTreeSet::new();
    for name in names {
        if name.is_empty()
            || !name
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
        {
            return Err(Error::Input(format!("invalid parameter name: {name}")));
        }
        if !unique.insert(name) {
            return Err(Error::Input(format!("duplicate parameter name: {name}")));
        }
    }
    Ok(())
}

pub fn parse_params(values: &[String]) -> Result<BTreeMap<String, String>, Error> {
    let mut params = BTreeMap::new();
    for value in values {
        let (name, raw) = value
            .split_once('=')
            .ok_or_else(|| Error::Input(format!("parameter must be NAME=VALUE: {value}")))?;
        validate_param_names(&[name.to_owned()])?;
        if params.insert(name.to_owned(), raw.to_owned()).is_some() {
            return Err(Error::Input(format!("parameter supplied twice: {name}")));
        }
    }
    Ok(params)
}

pub fn sha256_hex(bytes: impl AsRef<[u8]>) -> String {
    let digest = Sha256::digest(bytes.as_ref());
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

pub fn query_digest(sql: &str) -> String {
    sha256_hex(sql.trim().as_bytes())
}

pub fn database_path(database_url: &str) -> Result<PathBuf, Error> {
    let value = database_url
        .strip_prefix("sqlite://")
        .unwrap_or(database_url);
    if value.is_empty() || value == ":memory:" {
        return Err(Error::Input(
            "database URL must point to an existing SQLite file".into(),
        ));
    }
    if database_url.contains("://") && !database_url.starts_with("sqlite://") {
        return Err(Error::Input(
            "v0.1 supports SQLite URLs only (sqlite:///path/to/db.sqlite)".into(),
        ));
    }
    Ok(PathBuf::from(value))
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<serde_json::Value>>,
    pub row_count: usize,
    pub column_count: usize,
    pub truncated: bool,
}

pub fn execute_readonly(
    database_url: &str,
    sql: &str,
    params: &BTreeMap<String, String>,
    declared_params: Option<&[String]>,
    row_cap: usize,
    column_cap: usize,
) -> Result<QueryResult, Error> {
    let path = database_path(database_url)?;
    let connection = Connection::open_with_flags(
        &path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_URI,
    )
    .map_err(|e| Error::Database(format!("could not open SQLite database read-only: {e}")))?;

    let mut statement = connection.prepare(sql).map_err(|e| {
        let message = e.to_string();
        if message.contains("more than one statement") {
            Error::Policy("exactly one SQL statement is allowed".into())
        } else {
            Error::Database(format!("could not prepare query: {message}"))
        }
    })?;
    if !statement.readonly() {
        return Err(Error::Policy(
            "write or schema-changing SQL is never allowed".into(),
        ));
    }

    let mut sql_params = BTreeSet::new();
    for index in 1..=statement.parameter_count() {
        let Some(raw_name) = statement.parameter_name(index) else {
            return Err(Error::Policy(
                "anonymous SQL parameters are not allowed; use :name".into(),
            ));
        };
        let name = raw_name.trim_start_matches([':', '@', '$']);
        sql_params.insert(name.to_owned());
    }
    let supplied: BTreeSet<String> = params.keys().cloned().collect();
    if sql_params != supplied {
        return Err(Error::Input(format!(
            "query parameters do not match; expected [{}], received [{}]",
            sql_params.iter().cloned().collect::<Vec<_>>().join(", "),
            supplied.iter().cloned().collect::<Vec<_>>().join(", ")
        )));
    }
    if let Some(declared) = declared_params {
        let declared: BTreeSet<String> = declared.iter().cloned().collect();
        if declared != sql_params {
            return Err(Error::Policy(
                "template parameter declaration does not match its SQL".into(),
            ));
        }
    }
    for (name, value) in params {
        let token = format!(":{name}");
        let index = statement
            .parameter_index(&token)
            .map_err(|e| Error::Database(format!("could not inspect parameter {name}: {e}")))?
            .ok_or_else(|| Error::Input(format!("query has no parameter named {name}")))?;
        statement
            .raw_bind_parameter(index, value)
            .map_err(|e| Error::Database(format!("could not bind parameter {name}: {e}")))?;
    }

    let column_count = statement.column_count();
    if column_count > column_cap {
        return Err(Error::Policy(format!(
            "query exposes {column_count} columns; policy cap is {column_cap}"
        )));
    }
    let columns = statement
        .column_names()
        .iter()
        .map(|name| (*name).to_owned())
        .collect::<Vec<_>>();
    let mut cursor = statement.raw_query();
    let mut rows = Vec::new();
    let mut truncated = false;
    while let Some(row) = cursor
        .next()
        .map_err(|e| Error::Database(format!("could not read query result: {e}")))?
    {
        if rows.len() == row_cap {
            truncated = true;
            break;
        }
        let mut values = Vec::with_capacity(column_count);
        for index in 0..column_count {
            let value = row
                .get_ref(index)
                .map_err(|e| Error::Database(format!("could not read column: {e}")))?;
            values.push(match value {
                ValueRef::Null => serde_json::Value::Null,
                ValueRef::Integer(value) => value.into(),
                ValueRef::Real(value) => value.into(),
                ValueRef::Text(value) => String::from_utf8_lossy(value).into_owned().into(),
                ValueRef::Blob(value) => format!("base64:{}", STANDARD.encode(value)).into(),
            });
        }
        rows.push(values);
    }
    Ok(QueryResult {
        row_count: rows.len(),
        column_count,
        columns,
        rows,
        truncated,
    })
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReceiptPayload {
    pub schema_version: u8,
    pub receipt_id: Uuid,
    pub occurred_at: DateTime<Utc>,
    pub actor: String,
    pub query_kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template: Option<String>,
    pub query_sha256: String,
    pub parameter_names: Vec<String>,
    pub parameter_salt: String,
    pub parameter_sha256: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub database_sha256: Option<String>,
    pub approval: String,
    pub row_cap: usize,
    pub column_cap: usize,
    pub outcome: String,
    pub reason: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rows_returned: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub columns_returned: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub truncated: Option<bool>,
}

impl ReceiptPayload {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        actor: String,
        query_kind: String,
        template: Option<String>,
        sql: &str,
        params: &BTreeMap<String, String>,
        database_url: Option<&str>,
        approval: String,
        row_cap: usize,
        column_cap: usize,
        outcome: String,
        reason: String,
        result: Option<&QueryResult>,
    ) -> Self {
        let mut salt = [0_u8; 16];
        OsRng.fill_bytes(&mut salt);
        let parameter_salt = STANDARD.encode(salt);
        let encoded_params = serde_json::to_vec(params).expect("map serialization cannot fail");
        let mut parameter_hasher = Sha256::new();
        parameter_hasher.update(salt);
        parameter_hasher.update(encoded_params);
        let parameter_sha256 = parameter_hasher
            .finalize()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect();
        Self {
            schema_version: 1,
            receipt_id: Uuid::new_v4(),
            occurred_at: Utc::now(),
            actor,
            query_kind,
            template,
            query_sha256: query_digest(sql),
            parameter_names: params.keys().cloned().collect(),
            parameter_salt,
            parameter_sha256,
            database_sha256: database_url.map(sha256_hex),
            approval,
            row_cap,
            column_cap,
            outcome,
            reason,
            rows_returned: result.map(|result| result.row_count),
            columns_returned: result.map(|result| result.column_count),
            truncated: result.map(|result| result.truncated),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReceiptSignature {
    pub algorithm: String,
    pub public_key: String,
    pub value: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SignedReceipt {
    pub payload: ReceiptPayload,
    pub signature: ReceiptSignature,
}

pub struct ReceiptSigner(SigningKey);

impl ReceiptSigner {
    pub fn from_seed(seed: [u8; 32]) -> Self {
        Self(SigningKey::from_bytes(&seed))
    }

    pub fn from_base64(value: &str) -> Result<Self, Error> {
        let bytes = STANDARD
            .decode(value)
            .map_err(|_| Error::Receipt("signing key override is not valid base64".into()))?;
        let seed: [u8; 32] = bytes
            .try_into()
            .map_err(|_| Error::Receipt("signing key override must decode to 32 bytes".into()))?;
        Ok(Self::from_seed(seed))
    }

    pub fn load_or_create(profile: &str) -> Result<Self, Error> {
        if let Ok(value) = std::env::var("DB_RECEIPTS_SIGNING_KEY") {
            return Self::from_base64(&value);
        }
        let account = format!("{profile}:signing-key");
        let entry = keyring::Entry::new(KEYRING_SERVICE, &account)
            .map_err(|e| Error::Receipt(format!("could not open OS keychain: {e}")))?;
        match entry.get_password() {
            Ok(value) => Self::from_base64(&value),
            Err(keyring::Error::NoEntry) => {
                let mut seed = [0_u8; 32];
                OsRng.fill_bytes(&mut seed);
                entry.set_password(&STANDARD.encode(seed)).map_err(|e| {
                    Error::Receipt(format!("could not save signing key in OS keychain: {e}"))
                })?;
                Ok(Self::from_seed(seed))
            }
            Err(error) => Err(Error::Receipt(format!(
                "could not read signing key from OS keychain: {error}"
            ))),
        }
    }

    pub fn public_key_base64(&self) -> String {
        STANDARD.encode(self.0.verifying_key().as_bytes())
    }

    pub fn sign(&self, payload: ReceiptPayload) -> Result<SignedReceipt, Error> {
        let message = serde_json::to_vec(&payload)
            .map_err(|e| Error::Receipt(format!("could not serialize receipt: {e}")))?;
        let signature = self.0.sign(&message);
        Ok(SignedReceipt {
            payload,
            signature: ReceiptSignature {
                algorithm: "Ed25519".into(),
                public_key: self.public_key_base64(),
                value: STANDARD.encode(signature.to_bytes()),
            },
        })
    }
}

pub fn verify_receipt(receipt: &SignedReceipt) -> Result<(), Error> {
    if receipt.signature.algorithm != "Ed25519" {
        return Err(Error::Receipt(
            "unsupported receipt signature algorithm".into(),
        ));
    }
    let key_bytes: [u8; 32] = STANDARD
        .decode(&receipt.signature.public_key)
        .map_err(|_| Error::Receipt("receipt public key is not valid base64".into()))?
        .try_into()
        .map_err(|_| Error::Receipt("receipt public key has the wrong length".into()))?;
    let signature_bytes: [u8; 64] = STANDARD
        .decode(&receipt.signature.value)
        .map_err(|_| Error::Receipt("receipt signature is not valid base64".into()))?
        .try_into()
        .map_err(|_| Error::Receipt("receipt signature has the wrong length".into()))?;
    let key = VerifyingKey::from_bytes(&key_bytes)
        .map_err(|_| Error::Receipt("receipt public key is invalid".into()))?;
    let signature = Signature::from_bytes(&signature_bytes);
    let message = serde_json::to_vec(&receipt.payload)
        .map_err(|e| Error::Receipt(format!("could not serialize receipt: {e}")))?;
    key.verify(&message, &signature)
        .map_err(|_| Error::Receipt("receipt signature is invalid".into()))
}

pub fn write_receipt(directory: &Path, receipt: &SignedReceipt) -> Result<PathBuf, Error> {
    fs::create_dir_all(directory)
        .map_err(|e| Error::Receipt(format!("could not create receipt directory: {e}")))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(directory, fs::Permissions::from_mode(0o700))
            .map_err(|e| Error::Receipt(format!("could not secure receipt directory: {e}")))?;
    }
    let timestamp = receipt
        .payload
        .occurred_at
        .to_rfc3339_opts(SecondsFormat::Secs, true)
        .replace([':', '-'], "");
    let path = directory.join(format!("{timestamp}-{}.json", receipt.payload.receipt_id));
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options
        .open(&path)
        .map_err(|e| Error::Receipt(format!("could not create receipt: {e}")))?;
    let bytes = serde_json::to_vec_pretty(receipt)
        .map_err(|e| Error::Receipt(format!("could not serialize receipt: {e}")))?;
    file.write_all(&bytes)
        .and_then(|_| file.write_all(b"\n"))
        .map_err(|e| Error::Receipt(format!("could not write receipt: {e}")))?;
    file.sync_all()
        .map_err(|e| Error::Receipt(format!("could not sync receipt: {e}")))?;
    Ok(path)
}

pub fn load_database_url(profile: &str) -> Result<String, Error> {
    if let Ok(value) = std::env::var("DB_RECEIPTS_DATABASE_URL") {
        if !value.trim().is_empty() {
            return Ok(value);
        }
    }
    let account = format!("{profile}:database-url");
    let entry = keyring::Entry::new(KEYRING_SERVICE, &account)
        .map_err(|e| Error::Database(format!("could not open OS keychain: {e}")))?;
    entry.get_password().map_err(|error| match error {
        keyring::Error::NoEntry => Error::Database(format!(
            "no database URL stored for profile {profile}; run `db-receipts secret set`"
        )),
        other => Error::Database(format!(
            "could not read database URL from OS keychain: {other}"
        )),
    })
}

pub fn set_database_url(profile: &str, database_url: &str) -> Result<(), Error> {
    database_path(database_url)?;
    let account = format!("{profile}:database-url");
    let entry = keyring::Entry::new(KEYRING_SERVICE, &account)
        .map_err(|e| Error::Database(format!("could not open OS keychain: {e}")))?;
    entry
        .set_password(database_url)
        .map_err(|e| Error::Database(format!("could not save database URL in OS keychain: {e}")))
}

pub fn clear_database_url(profile: &str) -> Result<(), Error> {
    let account = format!("{profile}:database-url");
    let entry = keyring::Entry::new(KEYRING_SERVICE, &account)
        .map_err(|e| Error::Database(format!("could not open OS keychain: {e}")))?;
    match entry.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(error) => Err(Error::Database(format!(
            "could not clear database URL from OS keychain: {error}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn documented_example_config_parses() {
        let source = include_str!("../examples/db-receipts.toml");
        let config: Config = toml::from_str(source).unwrap();
        config.validate().unwrap();
        assert_eq!(config.template("open-orders").unwrap().row_cap, Some(50));
    }

    #[test]
    fn read_query_is_bounded_and_write_is_blocked() {
        let directory = tempdir().unwrap();
        let database = directory.path().join("test.sqlite");
        let connection = Connection::open(&database).unwrap();
        connection
            .execute_batch("CREATE TABLE items(id INTEGER, name TEXT); INSERT INTO items VALUES (1, 'fern'), (2, 'moss'), (3, 'lichen');")
            .unwrap();
        drop(connection);
        let params = BTreeMap::new();
        let result = execute_readonly(
            database.to_str().unwrap(),
            "SELECT id, name FROM items ORDER BY id",
            &params,
            Some(&[]),
            2,
            2,
        )
        .unwrap();
        assert_eq!(result.row_count, 2);
        assert!(result.truncated);
        let error = execute_readonly(
            database.to_str().unwrap(),
            "DELETE FROM items",
            &params,
            None,
            2,
            2,
        )
        .unwrap_err();
        assert!(matches!(error, Error::Policy(_)));
    }

    #[test]
    fn signed_receipt_detects_tampering_and_contains_no_values() {
        let mut params = BTreeMap::new();
        params.insert("account_id".into(), "secret-account".into());
        let signer = ReceiptSigner::from_seed([7; 32]);
        let payload = ReceiptPayload::new(
            "dev@example.test".into(),
            "template".into(),
            Some("open-orders".into()),
            "SELECT id FROM orders WHERE account_id = :account_id",
            &params,
            Some("sqlite:///private/db.sqlite"),
            "policy:open-orders".into(),
            50,
            6,
            "allowed".into(),
            "query completed".into(),
            None,
        );
        let receipt = signer.sign(payload).unwrap();
        verify_receipt(&receipt).unwrap();
        let json = serde_json::to_string(&receipt).unwrap();
        assert!(!json.contains("secret-account"));
        assert!(!json.contains("SELECT id"));
        assert!(!json.contains("private/db.sqlite"));
        let mut tampered = receipt;
        tampered.payload.actor = "other@example.test".into();
        assert!(verify_receipt(&tampered).is_err());
    }

    #[test]
    fn receipt_file_round_trips() {
        let directory = tempdir().unwrap();
        let signer = ReceiptSigner::from_seed([3; 32]);
        let payload = ReceiptPayload::new(
            "developer".into(),
            "novel".into(),
            None,
            "SELECT 1",
            &BTreeMap::new(),
            None,
            "human-challenge".into(),
            10,
            2,
            "allowed".into(),
            "query completed".into(),
            None,
        );
        let receipt = signer.sign(payload).unwrap();
        let path = write_receipt(directory.path(), &receipt).unwrap();
        let restored: SignedReceipt = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
        verify_receipt(&restored).unwrap();
    }
}
