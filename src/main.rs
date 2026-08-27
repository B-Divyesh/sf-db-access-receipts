use std::collections::BTreeMap;
use std::fs;
use std::io::{self, IsTerminal, Write};
use std::path::{Path, PathBuf};

use base64::{Engine as _, engine::general_purpose::STANDARD};
use clap::{Args, Parser, Subcommand};
use db_access_receipts::{
    Config, Error, ReceiptPayload, ReceiptSigner, SignedReceipt, clear_database_url,
    execute_readonly, initial_config, load_database_url, parse_params, query_digest,
    set_database_url, verify_receipt, write_receipt,
};
use rand::{Rng, thread_rng};
use serde_json::json;

#[derive(Parser)]
#[command(
    name = "db-receipts",
    version,
    about = "Bound read-only SQLite queries and issue signed audit receipts",
    long_about = "DB Access Receipts runs named or human-approved read-only SQLite queries. It enforces row and column caps, keeps credentials in the OS keychain, and signs a data-minimizing JSON receipt for every query attempt."
)]
struct Cli {
    /// Policy file to use.
    #[arg(long, global = true, default_value = "db-receipts.toml")]
    config: PathBuf,

    /// Emit machine-readable JSON.
    #[arg(long, global = true)]
    json: bool,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Create a commented policy file in the current directory.
    Init,
    /// Store or clear the database URL in the OS keychain.
    Secret(SecretArgs),
    /// List allowlisted query templates and their limits.
    Templates,
    /// Execute one bounded read query and write a signed receipt.
    Query(QueryArgs),
    /// Verify a receipt's Ed25519 signature without database access.
    Verify { receipt: PathBuf },
}

#[derive(Args)]
struct SecretArgs {
    #[command(subcommand)]
    command: SecretCommand,
}

#[derive(Subcommand)]
enum SecretCommand {
    /// Prompt for and store a SQLite URL. Passing the value may expose it in shell history.
    Set {
        #[arg(long)]
        database_url: Option<String>,
    },
    /// Report whether a URL is available without printing it.
    Status,
    /// Remove the stored database URL for this profile.
    Clear,
}

#[derive(Args)]
struct QueryArgs {
    /// Name of a reviewed template from the policy.
    #[arg(long, conflicts_with = "sql", required_unless_present = "sql")]
    template: Option<String>,

    /// Novel read-only SQL; requires a TTY human challenge.
    #[arg(
        long,
        conflicts_with = "template",
        required_unless_present = "template"
    )]
    sql: Option<String>,

    /// Bound parameter in NAME=VALUE form; repeat for multiple parameters.
    #[arg(long = "param")]
    params: Vec<String>,

    /// Person or service requesting access. Defaults to the local OS user.
    #[arg(long)]
    actor: Option<String>,

    /// Human approving novel SQL. A TTY challenge is still mandatory.
    #[arg(long)]
    approver: Option<String>,
}

fn main() {
    let cli = Cli::parse();
    if let Err(error) = run(&cli) {
        if cli.json {
            println!(
                "{}",
                json!({ "ok": false, "error": error.to_string(), "exit_code": error.exit_code() })
            );
        } else {
            eprintln!("error: {error}");
        }
        std::process::exit(error.exit_code());
    }
}

fn run(cli: &Cli) -> Result<(), Error> {
    match &cli.command {
        Command::Init => init(&cli.config, cli.json),
        Command::Secret(args) => secret(&cli.config, &args.command, cli.json),
        Command::Templates => templates(&cli.config, cli.json),
        Command::Query(args) => query(&cli.config, args, cli.json),
        Command::Verify { receipt } => verify(receipt, cli.json),
    }
}

fn init(path: &Path, json_output: bool) -> Result<(), Error> {
    if path.exists() {
        return Err(Error::Input(format!(
            "refusing to overwrite existing policy {}",
            path.display()
        )));
    }
    let profile = std::env::current_dir()
        .ok()
        .and_then(|path| {
            path.file_name()
                .map(|name| name.to_string_lossy().to_string())
        })
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "default".into())
        .replace(
            |ch: char| !(ch.is_ascii_alphanumeric() || ch == '-' || ch == '_'),
            "-",
        );
    fs::write(path, initial_config(&profile))
        .map_err(|e| Error::Input(format!("could not write policy {}: {e}", path.display())))?;
    if json_output {
        println!(
            "{}",
            json!({ "ok": true, "policy": path, "profile": profile })
        );
    } else {
        println!("Created {} for profile {profile}.", path.display());
        println!("Next: add a template, then run `db-receipts secret set`.");
    }
    Ok(())
}

fn secret(path: &Path, command: &SecretCommand, json_output: bool) -> Result<(), Error> {
    let config = Config::load(path)?;
    match command {
        SecretCommand::Set { database_url } => {
            let value = match database_url {
                Some(value) => value.clone(),
                None => rpassword::prompt_password("SQLite URL: ")
                    .map_err(|e| Error::Input(format!("could not read database URL: {e}")))?,
            };
            set_database_url(&config.profile, &value)?;
            output_status(json_output, "stored", &config.profile);
        }
        SecretCommand::Status => {
            load_database_url(&config.profile)?;
            output_status(json_output, "available", &config.profile);
        }
        SecretCommand::Clear => {
            clear_database_url(&config.profile)?;
            output_status(json_output, "cleared", &config.profile);
        }
    }
    Ok(())
}

fn output_status(json_output: bool, status: &str, profile: &str) {
    if json_output {
        println!(
            "{}",
            json!({ "ok": true, "status": status, "profile": profile })
        );
    } else {
        println!("Database URL {status} for profile {profile}.");
    }
}

fn templates(path: &Path, json_output: bool) -> Result<(), Error> {
    let config = Config::load(path)?;
    if json_output {
        let values = config
            .templates
            .iter()
            .map(|template| {
                let (rows, columns) = config.caps_for(Some(template));
                json!({
                    "name": template.name,
                    "description": template.description,
                    "params": template.params,
                    "row_cap": rows,
                    "column_cap": columns,
                })
            })
            .collect::<Vec<_>>();
        println!("{}", json!({ "ok": true, "templates": values }));
    } else if config.templates.is_empty() {
        println!("No templates are allowlisted in {}.", path.display());
        println!("Add a [[templates]] block, or use --sql for a human-approved novel read.");
    } else {
        println!("NAME\tROWS\tCOLS\tPARAMETERS\tDESCRIPTION");
        for template in &config.templates {
            let (rows, columns) = config.caps_for(Some(template));
            println!(
                "{}\t{}\t{}\t{}\t{}",
                template.name,
                rows,
                columns,
                template.params.join(","),
                template.description
            );
        }
    }
    Ok(())
}

fn query(path: &Path, args: &QueryArgs, json_output: bool) -> Result<(), Error> {
    let config = Config::load(path)?;
    let params = parse_params(&args.params)?;
    let actor = args.actor.clone().unwrap_or_else(default_actor);
    if actor.trim().is_empty() {
        return Err(Error::Input("actor cannot be empty".into()));
    }
    let signer = ReceiptSigner::load_or_create(&config.profile)?;

    let (sql, template_name, declared_params, query_kind, approval, caps) = if let Some(name) =
        &args.template
    {
        let template = match config.template(name) {
            Ok(template) => template,
            Err(error) => {
                emit_attempt_receipt(
                    &config,
                    &signer,
                    &actor,
                    "template",
                    Some(name.clone()),
                    "",
                    &params,
                    None,
                    "not-approved",
                    config.caps_for(None),
                    "denied",
                    &error.to_string(),
                    None,
                    json_output,
                )?;
                return Err(error);
            }
        };
        (
            template.sql.clone(),
            Some(template.name.clone()),
            Some(template.params.clone()),
            "template".to_owned(),
            format!("policy:{}", template.name),
            config.caps_for(Some(template)),
        )
    } else {
        let sql = args.sql.clone().expect("clap requires SQL or template");
        let caps = config.caps_for(None);
        let digest = query_digest(&sql);
        if !io::stdin().is_terminal() || !io::stderr().is_terminal() {
            let error = Error::Policy(
                "novel SQL requires an attached terminal; non-interactive request denied".into(),
            );
            emit_attempt_receipt(
                &config,
                &signer,
                &actor,
                "novel",
                None,
                &sql,
                &params,
                None,
                "not-approved",
                caps,
                "denied",
                &error.to_string(),
                None,
                json_output,
            )?;
            return Err(error);
        }
        let challenge = thread_rng().gen_range(100_000..=999_999);
        let approver = args.approver.clone().unwrap_or_else(default_actor);
        if approver.trim().is_empty() {
            return Err(Error::Input("approver cannot be empty".into()));
        }
        eprintln!("Novel query approval required");
        eprintln!("  Requesting actor: {actor}");
        eprintln!("  Human approver: {approver}");
        eprintln!("  SQL SHA-256: {digest}");
        eprintln!("  Limits: {} rows, {} columns", caps.0, caps.1);
        eprintln!("  SQL:\n{}", safe_for_terminal(&sql));
        if !params.is_empty() {
            eprintln!("  Bound parameters:");
            for (name, value) in &params {
                eprintln!("    {name}={}", safe_for_terminal(value));
            }
        }
        eprint!("Type {challenge} to approve this one query: ");
        io::stderr()
            .flush()
            .map_err(|e| Error::Input(format!("could not show approval prompt: {e}")))?;
        let mut entered = String::new();
        io::stdin()
            .read_line(&mut entered)
            .map_err(|e| Error::Input(format!("could not read approval: {e}")))?;
        if entered.trim() != challenge.to_string() {
            let error =
                Error::Policy("human approval challenge did not match; query denied".into());
            emit_attempt_receipt(
                &config,
                &signer,
                &actor,
                "novel",
                None,
                &sql,
                &params,
                None,
                "not-approved",
                caps,
                "denied",
                &error.to_string(),
                None,
                json_output,
            )?;
            return Err(error);
        }
        (
            sql,
            None,
            None,
            "novel".to_owned(),
            format!("human-challenge:{approver}"),
            caps,
        )
    };

    let database_url = match load_database_url(&config.profile) {
        Ok(value) => value,
        Err(error) => {
            emit_attempt_receipt(
                &config,
                &signer,
                &actor,
                &query_kind,
                template_name.clone(),
                &sql,
                &params,
                None,
                &approval,
                caps,
                "failed",
                &error.to_string(),
                None,
                json_output,
            )?;
            return Err(error);
        }
    };
    match execute_readonly(
        &database_url,
        &sql,
        &params,
        declared_params.as_deref(),
        caps.0,
        caps.1,
    ) {
        Ok(result) => {
            let path = emit_attempt_receipt(
                &config,
                &signer,
                &actor,
                &query_kind,
                template_name,
                &sql,
                &params,
                Some(&database_url),
                &approval,
                caps,
                "allowed",
                "query completed",
                Some(&result),
                false,
            )?;
            if json_output {
                println!(
                    "{}",
                    json!({ "ok": true, "receipt": path, "result": result })
                );
            } else {
                print_table(&result.columns, &result.rows);
                eprintln!(
                    "{} row(s), {} column(s){}",
                    result.row_count,
                    result.column_count,
                    if result.truncated {
                        " (row cap reached)"
                    } else {
                        ""
                    }
                );
                eprintln!("Receipt: {}", path.display());
            }
            Ok(())
        }
        Err(error) => {
            let outcome = if matches!(error, Error::Policy(_) | Error::Input(_)) {
                "denied"
            } else {
                "failed"
            };
            emit_attempt_receipt(
                &config,
                &signer,
                &actor,
                &query_kind,
                template_name,
                &sql,
                &params,
                Some(&database_url),
                &approval,
                caps,
                outcome,
                &error.to_string(),
                None,
                json_output,
            )?;
            Err(error)
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn emit_attempt_receipt(
    config: &Config,
    signer: &ReceiptSigner,
    actor: &str,
    query_kind: &str,
    template: Option<String>,
    sql: &str,
    params: &BTreeMap<String, String>,
    database_url: Option<&str>,
    approval: &str,
    caps: (usize, usize),
    outcome: &str,
    reason: &str,
    result: Option<&db_access_receipts::QueryResult>,
    announce_json: bool,
) -> Result<PathBuf, Error> {
    let payload = ReceiptPayload::new(
        actor.to_owned(),
        query_kind.to_owned(),
        template,
        sql,
        params,
        database_url,
        approval.to_owned(),
        caps.0,
        caps.1,
        outcome.to_owned(),
        reason.to_owned(),
        result,
    );
    let receipt = signer.sign(payload)?;
    let path = write_receipt(&config.receipt_dir, &receipt)?;
    if announce_json {
        eprintln!("{}", json!({ "receipt": path, "outcome": outcome }));
    } else if outcome != "allowed" {
        eprintln!("Receipt: {}", path.display());
    }
    Ok(path)
}

fn verify(path: &Path, json_output: bool) -> Result<(), Error> {
    let bytes = fs::read(path)
        .map_err(|e| Error::Receipt(format!("could not read receipt {}: {e}", path.display())))?;
    let receipt: SignedReceipt = serde_json::from_slice(&bytes)
        .map_err(|e| Error::Receipt(format!("invalid receipt JSON: {e}")))?;
    verify_receipt(&receipt)?;
    if json_output {
        println!(
            "{}",
            json!({
                "ok": true,
                "valid": true,
                "receipt_id": receipt.payload.receipt_id,
                "outcome": receipt.payload.outcome,
                "public_key": receipt.signature.public_key,
            })
        );
    } else {
        println!("Valid Ed25519 receipt {}", receipt.payload.receipt_id);
        println!("Outcome: {}", receipt.payload.outcome);
        println!("Actor: {}", receipt.payload.actor);
        println!("Occurred: {}", receipt.payload.occurred_at.to_rfc3339());
        println!("Public key: {}", receipt.signature.public_key);
    }
    Ok(())
}

fn print_table(columns: &[String], rows: &[Vec<serde_json::Value>]) {
    println!("{}", columns.join("\t"));
    for row in rows {
        println!(
            "{}",
            row.iter()
                .map(|value| match value {
                    serde_json::Value::Null => "NULL".to_owned(),
                    serde_json::Value::String(value) => value.replace(['\t', '\n'], " "),
                    other => other.to_string(),
                })
                .collect::<Vec<_>>()
                .join("\t")
        );
    }
}

fn default_actor() -> String {
    std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "unknown-local-user".into())
}

fn safe_for_terminal(value: &str) -> String {
    value
        .chars()
        .flat_map(|ch| {
            if ch == '\n' || ch == '\t' || !ch.is_control() {
                ch.to_string().chars().collect::<Vec<_>>()
            } else {
                ch.escape_default().collect::<Vec<_>>()
            }
        })
        .collect()
}

#[allow(dead_code)]
fn signing_key_for_docs() -> String {
    STANDARD.encode([0_u8; 32])
}
