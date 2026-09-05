# Verify SQLite query gating and signed receipts — verification 2

## Verdict

**FAIL — 5 findings and 8 untested or incompletely tested claim families.**

- Implementation reviewed: `cd36b77120f13acd98833dfe9e42d27a044f2811`
- Documentation reviewed: `ad40c63ca3e526217683e8a002602e55470db532`
- Live URL: <https://db-access-receipts.sociobot.in/>
- Verified: 5 September 2026 UTC

The live home, demo, legal pages, 404 page, and hashed assets match the local build byte for byte. Later commits through `ad40c63` only changed tests or documentation, so `cd36b77` is the live implementation candidate.

## Job, audience, and first action

Before scrolling in fresh 1440×1000 desktop and 390×844 phone contexts:

- Job: gate file-backed SQLite reads, block writes, and keep signed receipts.
- Audience: teams that let tools query production-adjacent SQLite data.
- First action: **Try it with sample data**, with the note that it loads a signed sample receipt.

The headline, audience sentence, action, and three facts are above the fold at both sizes. The phone has no horizontal page overflow. The desktop has the overlap recorded as Finding 5.

## Findings

### High

1. **The registered $39 Team Field Kit is absent from the product, and no paid entitlement can be used.** The live factory endpoint returns 303 to Dodo, and the hosted page returns 200 with **DB Access Receipts Team Field Kit**, **$39.00**, and a one-time license description. The product page has no checkout link, price, deliverable list, restore field, or unlocked download. Loading `/?license=qa-invalid-token` leaves the token in the address, stores no `sb_license:db-access-receipts` value, makes no verification request, and shows no license state. The actual verification endpoint correctly returns 200 with `valid:false` for the invalid token, but this does not prove a paid entitlement. The current `connect-src 'self'` CSP would block the required Sociobot verification call if the previous handler were restored without updating the header. The previous paid offer listed the already-researched deliverables: a 30-day pilot and daily review checklist, template review worksheet, incident evidence and receipt retention prompts, and updates for this major version. Registration is no longer a dependency, so the offer and its tested return, restore, verify, revocation, offline-cache, and download paths must be restored without gating free safety functions.

### Medium

2. **One of 17 declared claim commands fails from a clean checkout.** `sh tests/claim-mit-license.sh` exits 1. The package contains `LICENSE` and its full MIT terms, but the script requires an exact line equal to `MIT License`; the file begins with `Copyright (c) 2026 Sociobot (Param Factory)`. The public MIT statement is substantively true, but its required claim test is false. `npm test` does not run either shell claim command, so the normal suite does not expose this failure.

3. **Eight public or required claim families lack a complete claim test.** The gaps are: a correct one-use CLI challenge succeeding and not being reusable; multiple-statement refusal; omission of credentials from receipts; successful OS-keychain storage and retrieval; documented database-failure exit code 3; demo reset preserving pre-existing real preview data; valid-license return/restore/daily verification/revocation/offline-cache behavior; and delivery of the purchased Team Field Kit. Manual verification proved that demo reset preserves a real-data sentinel, but the declared demo test starts with the real key absent and therefore does not prove its own preservation statement. The challenge test only enters a wrong code. The other listed behaviors have no matching claim entry or complete sandbox assertion.

### Low

4. **Four phone controls are smaller than the required 44×44 px touch target.** At 390 px, the wordmark measures 170×40 px, the header Demo link 39×44 px, the footer Demo link 42×44 px, and the footer Terms link 41×44 px. Axe reports no violations, but the attached accessibility and design contracts require both dimensions to be at least 44 px.

5. **The third first-screen fact overlaps the hero illustration at 1440 px.** The left hero column is 598 px wide but its facts row has a 651 px scroll width. **Free and open source** ends at x=780.7 while the illustration begins at x=759.9, so roughly 20 px of the fact is painted under the image. The page itself does not report horizontal overflow, which is why the automated responsive check misses this clipping.

## Declared claim results

Every command in `.factory/claims.json` was run from a fresh clone after `npm ci`.

| Claim | Result |
| --- | --- |
| `cli-demo` | PASS |
| `install-from-clean-checkout` | PASS; isolated clone, Cargo root, installed binary, and JSON demo |
| `mit-license` | **FAIL**; command exited 1 after packaging |
| `cli-no-telemetry` | PASS |
| `templates-and-limits` | PASS |
| `novel-human-challenge` | PASS as written; incomplete for success and one-use semantics |
| `noninteractive-novel-denial` | PASS |
| `readonly-write-denial` | PASS |
| `column-cap` | PASS |
| `signed-attempts` | PASS |
| `offline-verification` | PASS |
| `receipt-minimization` | PASS; does not cover the separate public credential-omission statement |
| `json-exit-codes` | PASS for 0 and 2; does not cover documented database-failure code 3 |
| `no-plaintext-secret-fallback` | PASS; does not prove successful keychain retrieval |
| `demo-sandbox` | PASS as written; incomplete for preserving an existing real-data key |
| `local-only-browser` | PASS |
| `offline-demo` | PASS |

The strict result is 16 passing commands and 1 failing command. A failed or incomplete claim prevents PASS.

## Build, package, and clean consumer

From the fresh clone:

| Command | Result |
| --- | --- |
| `npm ci` | PASS; 0 vulnerabilities |
| `cargo fmt --all -- --check` | PASS |
| `cargo clippy --all-targets --all-features -- -D warnings` | PASS |
| `npm test` | PASS: 4 Rust unit, 12 Rust claim, 2 Rust integration, 3 Vitest, and 4 Playwright tests |
| `npm run build` | PASS; wrote `dist/site/` |
| `cargo build --release` | PASS |
| `cargo package` | PASS; 12 files, 29.1 KiB compressed |
| `sh tests/consumer-install.sh` | PASS |

The packaged crate was also installed into a separate Cargo root. Its `--help`, `--json demo`, and offline `--json verify` worked. The demo returned two realistic rows, printed an isolated temporary directory and receipt path, and produced a valid receipt. The receipt directory and file modes were 0700 and 0600.

## Live demo, normal, invalid, boundary, and recovery paths

The first click opens `/demo/` with the persistent **Demo — sample data, nothing is saved to your real data.** banner. It immediately shows **Allowed and signed**, actor `agent@northstar.example`, and `2/50 rows · 6 columns max`.

- A seeded ordinary key, `db-receipts:receipts`, remained unchanged through demo use, **Reset demo**, and **Start for real**.
- Demo state used `demo:db-receipts:receipts`. Reset restored the original sample. Start for real returned to `/` without the banner.
- Emptying the required actor field prevented submission and focused the actor input.
- A wrong novel-query code produced a signed denial with a useful recovery message.
- `FERN-42` then produced an allowed receipt.
- `DELETE FROM orders` remained denied after the correct browser-demo code.
- The ordinary preview created a receipt, retained its history across reload, cleared it, and restored its empty state.
- Arrow-key tab selection, Space on the theme button, receipt focus after async output, and keyboard skip-link focus worked.

No product request left `db-access-receipts.sociobot.in` during the home and demo flows. There were no product console or page errors. The only 404 console message occurred while deliberately requesting the expected unknown route; the styled page and HTTP 404 are correct, so it is not a defect.

## Accessibility, routes, privacy, offline, and performance

- Live axe WCAG 2 A/AA scans found zero violations on home, demo, privacy, terms, the unknown-route 404, and the dark demo.
- Every route has `lang=en`, one `h1`, one `main`, its own plain title, canonical metadata, and the product social image.
- The skip link is the first Tab stop and has a visible 3 px pollen-colour outline.
- Reduced motion changes smooth scrolling to `auto` and receipt animation to `0.01ms`.
- At 200% root text size, the demo retained its h1 and banner without page overflow.
- The service worker updated, controlled the page, and reloaded the populated demo offline with its offline status message.
- All crawled internal links and the GitHub source link returned 200. `/not-a-real-route-qa` returned the designed page with HTTP 404.
- Live responses include CSP with response-header `frame-ancestors`, Permissions Policy, `nosniff`, referrer policy, HSTS, and frame denial. Hashed assets are immutable for one year; the service worker is `no-cache`.
- `/opt/fleet/lib/verify-url.sh` passed with no errors.
- Mobile Lighthouse: Performance 100, Accessibility 100, Best Practices 100, SEO 100; LCP 1.244 s, CLS 0, and total blocking time 17 ms.
- Built initial JavaScript is 6,446 bytes and CSS is 11,695 bytes.

The site has no analytics, third-party fonts, or product-side tracking. Browser storage and clearing behavior match the privacy page. DB Access Receipts is a static site plus local CLI, so backend tenant isolation, restart persistence, health, and 429/`Retry-After` checks do not apply.

## Earlier finding disposition

| Review 1 item | Current disposition |
| --- | --- |
| False crates.io install command | Resolved; public source and clean consumer install work. |
| Missing CLI and browser demos | Resolved. |
| Broken checkout | Checkout registration is resolved, but the offer and entitlement were removed; open as Finding 1. |
| Missing claims manifest | Manifest exists; claim execution and coverage remain open as Findings 2 and 3. |
| Metaphor-heavy or inaccurate first-screen copy | Resolved. |
| Missing demo and real 404 routes | Resolved. |
| Incomplete metadata and shared structure | Resolved. |
| Missing demo and copy documents | Resolved. |
| Missing response policies | Resolved. |
| Missing immutable caching | Resolved. |
| Small touch targets and missing required-field help | Required-field help is resolved; target height improved, but four targets remain under 44 px in one dimension as Finding 4. |

The two older hardening findings for response policies and cache rules are both closed. SQLite-only scope, host keychain availability, and absent key rotation remain documented limits rather than defects, provided public keychain behavior gains a complete claim test.

## Evidence

- `/work/.evidence/verification-2-desktop.png`
- `/work/.evidence/verification-2-phone.png`
- `/work/.evidence/verification-2-checkout.png`
- `/work/.evidence/verify-url-2/verify.json`
- `/work/.evidence/lighthouse-2.json`

## Required next steps

1. Restore the registered $39 offer, its exact researched deliverables, legal copy, restore flow, and tested entitlement download. Update CSP `connect-src` for the Sociobot verification endpoint.
2. Fix the MIT claim command and ensure the main test command exercises both shell claims.
3. Add complete claim tests for all eight listed families.
4. Raise every phone control to 44×44 px and stop the desktop facts row from entering the illustration column.
