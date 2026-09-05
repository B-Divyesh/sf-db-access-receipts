# Demo contract

## Browser demo

- URL: `https://db-access-receipts.sociobot.in/demo/`
- First view: a populated allowed receipt for `agent@northstar.example`, two rows under a 50-row cap, and a six-column cap.
- Sample data: a reviewed `open-orders` SQLite read with account `acct_demo`; the browser view is an illustrative local receipt, not a database engine.
- Storage: browser demo receipts use only `demo:db-receipts:receipts`; the optional demo theme uses `demo:db-receipts:theme`.
- Reset: **Reset demo** deletes those demo keys and reloads the original sample.
- Exit: **Start for real** opens `/` without reading or copying demo keys.
- Safety: the demo makes no query, actor, account, or receipt request to another origin. It never reads or writes the ordinary preview key `db-receipts:receipts`.

The service worker caches the demo shell after its first visit. The demo remains usable offline after that visit.

## CLI demo

- Command: `db-receipts demo`
- Sample data: `examples/demo-orders.sql` creates three realistic order records; `examples/demo-policy.toml` supplies the named `open-orders` template.
- Isolation: the command makes a new `db-access-receipts-demo-<uuid>` directory beneath the operating system temporary directory. It creates its database, generated policy, and receipt there.
- Output: it prints the directory, two returned sample rows, a signed receipt path, and the command to verify that receipt.
- Reset: remove the printed temporary directory when finished. The command never opens a configured production database or OS-keychain secret.

Claim tests use only the browser demo URL or the CLI command and bundled sample files. See `.factory/claims.json`.
