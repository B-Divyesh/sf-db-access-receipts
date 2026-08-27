# DB Access Receipts

DB Access Receipts is a local, read-only SQLite query gate for teams letting tools or agents inspect production-adjacent data. It keeps your existing database client and credentials local while adding named query templates, bounded results, explicit human approval for novel SQL, and Ed25519-signed JSON audit receipts that contain no result data.

The public surface starts at `0.1.0`. There is no telemetry and no hosted data plane.

## Install

Build the single binary with a current Rust toolchain:

```sh
cargo install --path .
db-receipts --help
```

Factory release artifacts can be prepared with `cargo package`; registry publishing is intentionally left to the factory.

## Usage

Initialize a policy in the current directory:

```sh
db-receipts init
```

Store the database URL in your OS keychain, then add a named template to `db-receipts.toml`:

```sh
db-receipts secret set
```

```toml
version = 1
receipt_dir = ".db-receipts/receipts"
default_row_cap = 100
default_column_cap = 12

[[templates]]
name = "open-orders"
description = "Open orders for one account"
sql = "SELECT id, status, created_at FROM orders WHERE account_id = :account_id"
params = ["account_id"]
row_cap = 50
column_cap = 6
```

Run an allowlisted template. Values are bound parameters, never interpolated into SQL:

```sh
db-receipts query --template open-orders --param account_id=acct_123
```

Run a novel read query. In a terminal, the CLI displays the SQL hash and limits and requires typing a one-use challenge. In CI or any non-interactive session, novel SQL is denied and still receives a signed denial receipt.

```sh
db-receipts query --sql "SELECT name FROM sqlite_schema" --actor dev@company
```

Use JSON output for scripts and verify receipts offline:

```sh
db-receipts --json query --template open-orders --param account_id=acct_123
db-receipts verify .db-receipts/receipts/<receipt-id>.json
db-receipts templates
```

Exit codes are `0` for success, `2` for policy denial or invalid input, `3` for database/query failure, and `4` for receipt/signature failure. `--json` writes machine-readable events to stdout; query result rows are deliberately written to stdout only after policy approval. Receipts record parameter names and a salted digest, not parameter values or result data.

### Headless test mode

The normal secret path is the OS keychain. For containers and CI only, set `DB_RECEIPTS_DATABASE_URL` and `DB_RECEIPTS_SIGNING_KEY` (base64-encoded 32-byte Ed25519 seed). These explicit overrides make reproducible testing possible without silently falling back to plaintext files.

## Safety model

- SQLite is opened read-only, and each prepared statement must report itself read-only.
- Exactly one statement is accepted; writes and trailing statements are denied.
- Named templates declare their parameter names and limits.
- Novel SQL requires an attached terminal and a randomized human challenge.
- Column caps reject over-broad results. Row caps stop iteration and mark truncation.
- A signed receipt is attempted for successful, denied, and failed queries.
- Receipts exclude raw SQL, parameter values, credentials, and returned cells by default.

This is an approval and evidence layer, not credential rotation, natural-language-to-SQL, an agent framework, or a substitute for database-side authorization.

## Develop and verify

```sh
cargo test
cargo build --release
npm ci
npm test
npm run build:site  # outputs dist/site/index.html
npm run build       # same production site build
```

The static site is in `site/`, uses Vite with vanilla TypeScript, and includes an offline-capable interactive policy/receipt walkthrough. It sends no query or demo data anywhere. The only network call beyond same-origin assets is an optional Sociobot license verification after a buyer supplies a license.

## Deploy

Deploy `dist/site/` at `https://db-access-receipts.sociobot.in`. The CLI itself is released separately as a single Rust binary by the factory. No infrastructure, DNS, billing credentials, or product IDs are stored in this repository.

## License

MIT — see [LICENSE](LICENSE).
