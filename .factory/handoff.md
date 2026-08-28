# DB Access Receipts v0.1 handoff

## Independent verification verdict — PASS

Candidate `7bbfcf546b4b97801dd9b99c16d5e81e0c7df879` was independently verified on 2026-08-27/28 UTC. The live deployment at <https://db-access-receipts.sociobot.in/> is available and byte-for-byte matches the locally built home page, JS, CSS, service worker, privacy page, and terms page. Full evidence, exact commands, CLI boundary exercises, browser/PWA/a11y checks, Lighthouse results, and low-severity deployment findings are in [verification.md](verification.md).

Result summary: clean `npm ci`, lint/type/build/package checks, 10 automated tests, a clean-consumer packed install, real SQLite allow/deny/receipt/tamper exercises, desktop/mobile keyboard testing, axe (0 serious/critical), production header/privacy/request review, offline reload, and live Lighthouse (99 performance / 100 accessibility) passed. Low-severity follow-ups are immutable caching for hashed assets and restrictive response-policy headers; neither blocks the verified safety model.

## What shipped

- A Rust 2024 single-binary CLI, `db-receipts`, with useful `--help`, stable exit codes, and `--json` output.
- `init`, `secret set/status/clear`, `templates`, `query`, and offline `verify` commands.
- Named, parameterized SQLite query templates with enforced row/column caps.
- Mandatory attached-TTY challenge for novel SQL, with separate requester and human-approver attribution. There is no non-interactive override.
- SQLite opened with `SQLITE_OPEN_READ_ONLY`, one prepared statement only, and a `readonly()` check before execution. Writes and schema changes are denied even if a person tries to approve them.
- Ed25519-signed JSON receipts for allowed, denied, and failed query attempts. Receipts contain query/database hashes, salted parameter digests, names, limits, actor, approval path, counts, and outcome; they exclude raw SQL, database URLs, parameter values, and result cells.
- Database URL and generated signing seed stored in the OS keychain. Explicit environment overrides exist only for CI/headless test environments.
- A Vite/vanilla TypeScript landing and documentation site with an interactive local-only allow/deny walkthrough, empty/error/offline states, keyboard tab behavior, dark/light treatment, `/privacy/`, and `/terms/`.
- One-time $39 Team Field Kit purchase and restore flow through the Sociobot API. Cached valid licenses unlock optimistically; verification runs in the background no more than daily. No product ID or payment provider is embedded.
- Versioned offline service-worker shell, robots/sitemap files, and an original botanical query-fern hero with responsive 23 KB and 79 KB WebP sources.

## Run and verify

```sh
# CLI quality gates
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo build --release
cargo package

# Static site quality gates
npm ci
npm test
npm run build:site
```

The work-order build command is `npm run build:site`. It produces `dist/site/index.html` plus the privacy and terms routes. The equivalent `npm run build` alias produces the same output. Prepare the registry artifact with `cargo package`; publishing is intentionally left to the factory.

Automated coverage includes four Rust unit tests, two real binary integration tests, and four TypeScript policy/cache tests. The binary tests execute a real SQLite template query, verify its signed receipt, confirm receipt data minimization, and confirm that a headless novel query is denied but still receipted.

Browser verification used production output at 390×844 and 1440×1000:

- Axe WCAG 2 A/AA + best-practice scan: zero violations on `/`, `/privacy/`, and `/terms/`.
- One `<h1>`, one `<main>`, `lang`, title, image alternatives, and no horizontal overflow on all three routes.
- Keyboard arrow navigation between query modes, allow and deny paths, offline status, return-license URL stripping, cached unlock, and service-worker offline navigation all passed.
- No page errors or console errors.
- Lighthouse mobile: Performance 100, Accessibility 100, Best Practices 100, SEO 100; LCP 1.36 s, CLS 0, total blocking time 55 ms.
- Production payload: 8.6 KB JS, 10.7 KB CSS, 23 KB mobile hero (79 KB large hero); no webfont payload.
- `npm audit --omit=dev`: zero production vulnerabilities.

## Known gaps and decisions

- v0.1 supports file-backed SQLite only. PostgreSQL/MySQL adapters are intentionally deferred; the read-only database handle makes the first release's safety claim directly testable. The public config and receipt schema leave room for future adapters.
- OS-keychain behavior depends on the host's available native keychain service. Containers should use the documented CI environment overrides; the CLI never silently writes plaintext credentials or signing keys.
- Receipt signing keys are created per policy profile, but v0.1 does not yet include a key-rotation command or organization trust registry. Each receipt carries its public key and verifies offline.
- The Team Field Kit checkout becomes purchasable after the factory registers the slug with Sociobot billing. The implementation already targets the production contract endpoint.
- The browser walkthrough is explanatory and deliberately not a SQL engine. Enforcement claims refer to the tested Rust CLI.

## Recommended next steps

1. Run a 30-day pilot on a copied or production-adjacent SQLite database and review denial receipts daily.
2. Add a PostgreSQL adapter only with server-side read-only transactions and equivalent multi-statement tests.
3. Add explicit signing-key rotation with a signed predecessor/successor chain before organization-wide retention policies depend on one key.
