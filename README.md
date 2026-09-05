# DB Access Receipts

DB Access Receipts gates file-backed SQLite reads for teams that let tools inspect production-adjacent data. It keeps the database local, accepts reviewed templates or a terminal approval, and writes signed JSON receipts without result data.

The core CLI is MIT-licensed. It has no hosted data plane or account requirement.

## Install

Clone the public source and install the single binary with a current Rust toolchain:

```sh
git clone https://github.com/B-Divyesh/sf-db-access-receipts.git
cd sf-db-access-receipts
cargo install --path . --locked
db-receipts --help
```

The crate is not published to crates.io. Do not use `cargo install db-access-receipts`.

## Try the bundled sample

Run this with no database setup:

```sh
db-receipts demo
```

It creates a small sample SQLite database and signed receipt in a new temporary directory. The command prints that directory and the receipt path. Verify the receipt later without database access:

```sh
db-receipts verify /tmp/db-access-receipts-demo-…/receipts/….json
```

The bundled sample policy and SQL seed live in `examples/demo-policy.toml` and `examples/demo-orders.sql`.

## Use with your SQLite file

Initialize a policy in an empty working directory:

```sh
db-receipts init
```

Store a file-backed SQLite URL with your operating system keychain:

```sh
db-receipts secret set
```

The command fails if the keychain is unavailable; it does not write a plaintext fallback. In containers and CI only, use the explicit `DB_RECEIPTS_DATABASE_URL` and `DB_RECEIPTS_SIGNING_KEY` overrides.

Add a reviewed template to `db-receipts.toml`:

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

Run a named template with bound values:

```sh
db-receipts query --template open-orders --param account_id=acct_123 --actor analyst@team
```

Run novel SQL only in an attached terminal. The CLI shows its SQL hash and limits, then requires a one-use human challenge. A non-interactive novel query is denied and receives a receipt.

```sh
db-receipts query --sql "SELECT name FROM sqlite_schema" --actor agent@company --approver dev@company
```

Use JSON for scripts and verify a receipt offline:

```sh
db-receipts --json query --template open-orders --param account_id=acct_123
db-receipts --json verify .db-receipts/receipts/receipt.json
db-receipts templates
```

Exit codes are `0` for success, `2` for policy denial or invalid input, `3` for database/query failure, and `4` for receipt/signature failure.

## Safety boundary

- SQLite opens read-only. Write SQL and multiple statements are refused.
- Named templates require matching parameters and declared row and column caps.
- Row caps truncate output. Column caps reject over-broad output.
- Successful, denied, and failed attempts receive Ed25519-signed receipts.
- Receipts omit raw SQL, parameter values, database paths, credentials, and returned cells.
- Receipt verification needs no database connection. A changed receipt fails verification.

This is not credential rotation, natural-language-to-SQL, database authorization, or an agent framework. Version 0.1 supports file-backed SQLite only.

## Browser demo and privacy

Open [the demo](https://db-access-receipts.sociobot.in/demo/) or `/demo/` on a local build. It loads a populated sample receipt in a separate `demo:` local-storage namespace. Reset demo discards that sample namespace; Start for real returns to the ordinary local preview without reading it.

The browser sample sends no query or receipt data away from the product site. It works offline after the first visit. Read the [privacy policy](https://db-access-receipts.sociobot.in/privacy/) and [terms](https://db-access-receipts.sociobot.in/terms/).

## Develop, test, and package

```sh
npm ci
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
npm test
npm run build
cargo build --release
cargo package
```

`npm test` runs Rust unit and CLI claim tests, TypeScript policy tests, and Playwright browser tests. `npm run build` writes the static site to `dist/site/`. Every public behavior is listed with its isolated command in `.factory/claims.json`.

Test the release artifact in a clean consumer directory before publishing:

```sh
cargo package
mkdir -p /tmp/db-receipts-consumer
cargo install --path target/package/db-access-receipts-0.1.1 --root /tmp/db-receipts-consumer --locked
/tmp/db-receipts-consumer/bin/db-receipts demo
```

The factory owns registry publishing. A future crates.io release should publish the package before changing the install instructions.

## Deploy

Build `dist/site/` and deploy it at `https://db-access-receipts.sociobot.in`. The static site configuration includes restrictive response headers, cache rules for fingerprinted assets, offline caching, and a designed 404 response. The factory controls deployment, DNS, and any future billing registration.

## License

MIT — see [LICENSE](LICENSE).
