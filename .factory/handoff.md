# DB Access Receipts v0.1.1 handoff

## Release identity

- Implementation SHA: `cd36b77120f13acd98833dfe9e42d27a044f2811`
- Documentation: this handoff is committed after the implementation; the final documentation SHA is the repository `HEAD` at handoff.
- Live URL: <https://db-access-receipts.sociobot.in/>
- Deployment: Azure Static Web App `sf-db-access-receipts`, one static site and no product backend.

## What changed

- Added `db-receipts demo`. It builds the shipped SQLite sample in a new temporary directory, runs the reviewed template, signs a receipt, and prints where to inspect it.
- Replaced the false crates.io install instruction with the tested public source-checkout command: clone the repository, then run `cargo install --path . --locked`.
- Added a one-click `/demo/` browser sandbox. It loads a populated receipt immediately and uses only `demo:` local-storage keys. It has persistent **Reset demo** and **Start for real** controls.
- Added the self-hosted CLI terminal recording, demo documentation, sample policy and data, and a real `/404.html` response.
- Rewrote public copy around file-backed SQLite, the target team, the first sample action, and three short facts. The copy audit is in `.factory/copy-audit.md`.
- Added per-route titles, canonical/OG/Twitter metadata, social and touch assets, consistent header/footer, an apple-touch icon, sitemap entries, and a product-styled 404 page.
- Added `staticwebapp.config.json` with CSP/frame protection, Permissions Policy, immutable asset caching, service-worker no-cache, and a 404 response override.
- Raised visible header/footer targets to at least 44 px and added required-field explanations.
- Added 17 outcome-based claim checks in `.factory/claims.json`, including the clean install, package license, browser sandbox, privacy, offline, and CLI safety paths.

## Verification

Run from a clean checkout:

```sh
npm ci
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
npm test
npm run build
cargo build --release
cargo package
sh tests/consumer-install.sh
```

The claim commands are listed individually in `.factory/claims.json`. `npm test` currently passes 4 Rust unit tests, 12 CLI claim tests, 2 CLI integration tests, 3 TypeScript tests, and 4 Playwright tests. The claim suite also exercises a fresh pseudo-terminal, read/write boundaries, cap behavior, signature tampering, receipt minimization, browser request logging, isolated demo reset, and offline reload.

The live implementation was cold-checked after deployment:

- Fresh desktop (1440×1000) and phone (390×844) pages showed **Gate SQLite reads with signed receipts**, the team audience sentence, and **Try it with sample data** before scrolling. Neither had console errors or horizontal overflow.
- `/demo` loaded the populated signed receipt, showed the persistent demo banner, reset to the same sample, stayed in the `demo:` namespace, and worked offline after its first visit.
- Live axe WCAG 2 A/AA scan found no violations. Local browser tests scan every route in light treatment and the demo in dark treatment.
- `/opt/fleet/lib/verify-url.sh` passed live: HTTP 200, title, `lang=en`, one h1, main landmark, alternatives, and no console errors.
- Live mobile Lighthouse: Performance 100, Accessibility 100, Best Practices 100, SEO 100; LCP 0.915 s, CLS 0, total blocking time 0 ms.
- Live `/not-a-real-route-qa` returns the styled document with HTTP 404. Live `/demo` returns the demo with HTTP 200.
- Live home responses have CSP, Permissions Policy, frame protection, `nosniff`, and referrer policy. The fingerprinted JavaScript returns `Cache-Control: public, max-age=31536000, immutable`.
- Production initial JavaScript is 6.45 KB (2.82 KB gzip) and CSS is 11.70 KB (3.43 KB gzip). No third-party runtime request is present.

## Review finding disposition

| Review 1 finding | Disposition |
| --- | --- |
| False `cargo install db-access-receipts` command | Resolved with public clone + `cargo install --path . --locked`; clean-consumer regression added. |
| Missing CLI and web demo | Resolved with `db-receipts demo`, `/demo/`, isolated storage, reset/exit controls, sample files, recording, and demo docs. |
| Checkout returned 404 | Resolved honestly by withdrawing the unregistered paid offer, restore flow, and license claims. No broken checkout link is shipped. |
| No claims manifest | Resolved with 17 isolated outcome-based claim commands. |
| Metaphor-heavy and inaccurate copy | Resolved; copy names SQLite, the audience, first sample action, and scope. |
| Missing demo and 404 routes | Resolved; `/demo`, `/demo/`, and designed 404 response work. |
| Incomplete metadata and shared structure | Resolved across home, demo, legal, and 404 pages. |
| Missing demo/copy documents | Resolved with `.factory/demo.md` and `.factory/copy-audit.md`. |
| Missing response policies | Resolved in static deployment configuration and verified live. |
| Missing immutable asset caching | Resolved for fingerprinted assets and verified live. |
| Small touch targets and missing required-field explanation | Resolved and tested at a phone viewport. |

## Known gaps and named dependencies

- Version 0.1 supports file-backed SQLite only. PostgreSQL and MySQL adapters need equivalent read-only transaction and multi-statement protection before release.
- OS-keychain access depends on a native keychain on the user host. A headless container fails safely rather than creating a plaintext secret. CI may use the documented explicit overrides.
- Key rotation and organization trust registry are still outside v0.1.
- The researched one-time rollout kit is deliberately not offered because its Sociobot billing product is not registered. The factory must register and live-test that product-specific checkout before a paid page, license storage, or price claim is restored.

## Next steps

1. Run the CLI against a copied production-adjacent SQLite database and review denial receipts during a pilot.
2. Add key rotation before organizations depend on one signing key long-term.
3. Register and verify a product-specific Sociobot checkout only when the rollout kit is ready to ship.
