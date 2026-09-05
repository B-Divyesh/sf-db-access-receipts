# DB Access Receipts v0.1 handoff

## Review 1 verdict — FAIL

The strict audit on 2026-09-05 reviewed implementation `7bbfcf546b4b97801dd9b99c16d5e81e0c7df879` at live URL <https://db-access-receipts.sociobot.in/> and documentation commit `c833ba5f9b3583697b4935f5dd97b22939ffc2f1`. The live build matches the implementation candidate byte for byte.

Result: **FAIL with 11 findings and 14 untested public claim families.** The three high-severity failures are the unavailable displayed Cargo install command, the missing required live/CLI demo contract, and the checkout link returning HTTP 404. The complete evidence, passing core CLI exercises, browser/accessibility/performance results, and disposition of every earlier finding are in [review-1.md](review-1.md).

No product code was changed during review. `npm ci`, formatting, clippy, 10 automated tests, production build, release build, packaging, clean-consumer installation, and the core SQLite safety exercises passed. Lighthouse scored 100 in all four measured categories, and axe found zero violations, but those passes do not override the findings or untested claims.

## Earlier independent verification — superseded

Candidate `7bbfcf546b4b97801dd9b99c16d5e81e0c7df879` was independently verified under the earlier acceptance standard on 2026-08-27/28 UTC. Full evidence is in [verification.md](verification.md). Its PASS label is superseded by Review 1 because that report itself listed two open low-severity findings, while Review 1 requires zero findings and zero untested claims.

Earlier result summary: clean `npm ci`, lint/type/build/package checks, 10 automated tests, a clean-consumer packed install, real SQLite allow/deny/receipt/tamper exercises, desktop/mobile keyboard testing, axe (0 serious/critical), production header/privacy/request review, offline reload, and live Lighthouse passed. The immutable-cache and response-policy findings remain open in Review 1.

## What shipped

- A Rust 2024 single-binary CLI, `db-receipts`, with useful `--help`, stable exit codes, and `--json` output.
- `init`, `secret set/status/clear`, `templates`, `query`, and offline `verify` commands.
- Named, parameterized SQLite query templates with enforced row/column caps.
- Mandatory attached-TTY challenge for novel SQL, with separate requester and human-approver attribution. There is no non-interactive override.
- SQLite opened with `SQLITE_OPEN_READ_ONLY`, one prepared statement only, and a `readonly()` check before execution. Writes and schema changes are denied even if a person tries to approve them.
- Ed25519-signed JSON receipts for allowed, denied, and failed query attempts. Receipts contain query/database hashes, salted parameter digests, names, limits, actor, approval path, counts, and outcome; they exclude raw SQL, database URLs, parameter values, and result cells.
- Database URL and generated signing seed stored in the OS keychain. Explicit environment overrides exist only for CI/headless test environments.
- A Vite/vanilla TypeScript landing and documentation site with an interactive local-only allow/deny walkthrough, empty/error/offline states, keyboard tab behavior, dark/light treatment, `/privacy/`, and `/terms/`.
- A $39 Team Field Kit purchase and restore UI targeting the Sociobot API. Cached valid licenses unlock optimistically and verification runs in the background no more than daily, but Review 1 confirms the live checkout currently returns HTTP 404.
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
- The Team Field Kit checkout is not registered: the live buy link returns HTTP 404. Do not present it as purchasable until the product-specific checkout succeeds.
- The browser walkthrough is explanatory and deliberately not a SQL engine. Enforcement claims refer to the tested Rust CLI.

## Recommended next steps

1. Run a 30-day pilot on a copied or production-adjacent SQLite database and review denial receipts daily.
2. Add a PostgreSQL adapter only with server-side read-only transactions and equivalent multi-statement tests.
3. Add explicit signing-key rotation with a signed predecessor/successor chain before organization-wide retention policies depend on one key.
