# DB Access Receipts v0.1.1 handoff

## Verification 2 status

**FAIL — 5 findings and 8 untested or incompletely tested claim families.** See [`.factory/verification-2.md`](verification-2.md).

This independent pass changed no product code. The live site is byte-for-byte the `cd36b77` implementation. The free CLI, browser demo, build, package, offline path, privacy boundary, accessibility scans, headers, caching, and Lighthouse checks work. However, the now-registered $39 Team Field Kit is absent from the product and has no license or entitlement flow. One declared claim command fails, eight claim families remain incomplete, four phone targets are under 44×44 px, and the third desktop first-screen fact overlaps the illustration.

## Release identity

- Implementation SHA: `cd36b77120f13acd98833dfe9e42d27a044f2811`
- Verification documentation SHA: `39eb4a54c5f5800c920780c219e22856c303023e`
- Package-license regression SHA: `76d73e5d046d36a28fdb452cb2fb80cc2eca73d7`
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
- Raised visible header/footer target heights and added required-field explanations. Verification 2 found four targets still narrower or shorter than 44 px.
- Added 17 claim commands in `.factory/claims.json`, including the clean install, package license, browser sandbox, privacy, offline, and CLI safety paths. Verification 2 found one failed command and incomplete coverage in eight families.

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

The claim commands are listed individually in `.factory/claims.json`. `npm test` currently passes 4 Rust unit tests, 12 CLI claim tests, 2 CLI integration tests, 3 TypeScript tests, and 4 Playwright tests. It does not run the two shell claim commands; the MIT shell claim currently fails. The suite exercises a fresh pseudo-terminal, read/write boundaries, cap behavior, signature tampering, receipt minimization, browser request logging, isolated demo reset, and offline reload, with the incomplete assertions listed in Verification 2.

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
| Checkout returned 404 | The checkout is now registered and healthy, but the paid offer and entitlement flow remain absent; open in Verification 2. |
| No claims manifest | Manifest exists, but one command fails and eight claim families are incomplete; open in Verification 2. |
| Metaphor-heavy and inaccurate copy | Resolved; copy names SQLite, the audience, first sample action, and scope. |
| Missing demo and 404 routes | Resolved; `/demo`, `/demo/`, and designed 404 response work. |
| Incomplete metadata and shared structure | Resolved across home, demo, legal, and 404 pages. |
| Missing demo/copy documents | Resolved with `.factory/demo.md` and `.factory/copy-audit.md`. |
| Missing response policies | Resolved in static deployment configuration and verified live. |
| Missing immutable asset caching | Resolved for fingerprinted assets and verified live. |
| Small touch targets and missing required-field explanation | Required-field help is resolved. Four targets remain under 44 px in one dimension; open in Verification 2. |

## Known gaps and named dependencies

- Version 0.1 supports file-backed SQLite only. PostgreSQL and MySQL adapters need equivalent read-only transaction and multi-statement protection before release.
- OS-keychain access depends on a native keychain on the user host. A headless container fails safely rather than creating a plaintext secret. CI may use the documented explicit overrides.
- Key rotation and organization trust registry are still outside v0.1.
- The Sociobot product is now registered in Live and Test. The live checkout redirects to Dodo and shows the correct DB Access Receipts Team Field Kit at $39 once. The product still needs its exact researched deliverables, buy and restore controls, license return handling, verification, revocation/offline behavior, and entitled download restored and tested. The current CSP must allow the product-specific Sociobot verification request.
- `sh tests/claim-mit-license.sh` fails because it searches for a heading that the packaged MIT text does not contain. `npm test` does not run this shell claim.
- Eight claim families listed in `.factory/verification-2.md` need complete sandbox coverage.
- Four phone controls miss the 44×44 px target minimum, and the third desktop fact overlaps the hero illustration at 1440 px.

## Next steps

1. Restore and test the registered $39 Team Field Kit purchase and entitlement path without changing the free safety features.
2. Fix the failed MIT claim command and add complete tests for every retained public claim.
3. Correct the phone touch targets and desktop first-screen overlap.
4. Run the CLI against a copied production-adjacent SQLite database and review denial receipts during a pilot.
5. Add key rotation before organizations depend on one signing key long-term.
