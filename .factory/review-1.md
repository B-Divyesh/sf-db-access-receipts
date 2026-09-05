# Review 1: Gate SQLite queries and issue signed receipts

## Verdict

**FAIL — 11 findings, including 3 high-severity findings, and 14 untested public claim families.**

The implementation reviewed is `7bbfcf546b4b97801dd9b99c16d5e81e0c7df879`. The documentation commit is `c833ba5f9b3583697b4935f5dd97b22939ffc2f1`. Commit `c833ba5` changes only `.factory/handoff.md` and `.factory/verification.md`. The live home, JavaScript, CSS, service worker, privacy page, and terms page match the local build byte for byte, so the live implementation candidate remains `7bbfcf5`.

Live URL: <https://db-access-receipts.sociobot.in/>  
Review date: 2026-09-05 UTC

## First screen

- Job: allow named or human-approved SQLite reads and issue signed receipts.
- Audience: teams that let tools or agents query production-adjacent SQLite databases.
- First action shown: **Try the approval path**.

All three are inferable before scrolling on desktop and phone. The audience is not stated directly in the first-screen sentence, the action does not say it uses sample data, and the first click only scrolls to an empty receipt state.

## Findings

### High

1. **The displayed install command does not work.** The first screen tells visitors to run `cargo install db-access-receipts`. From a clean consumer directory, Cargo exited 101 with `could not find db-access-receipts in registry crates-io`. `cargo search db-access-receipts` returned no package. The source-based README command and a packed local install work, but that does not make the live command true.

2. **The required one-click demo is missing from both product surfaces.** The live action is **Try the approval path**, not **Try it with sample data**, and it only scrolls. A second click is needed to produce output. `/demo` and `/?demo=1` render the ordinary landing page with the ordinary title. There is no persistent “Demo — sample data, nothing is saved” label, **Reset demo**, or **Start for real**. The installed CLI exits 2 for `db-receipts demo`, and the landing page has no recording of the real binary. `.factory/demo.md` is absent. The produced browser receipt is realistic and local, but it does not satisfy the demo entry and mode contract.

3. **The paid purchase path is broken.** The live **Buy the Team Field Kit** link returns HTTP 404 from the product-specific Sociobot checkout endpoint. An invalid-token verification request returns a normal 200 `{valid:false}` response, and the restore UI handles that response, but a visitor cannot buy the advertised $39 kit.

### Medium

4. **There is no claims manifest or claim-tagged browser/CLI suite.** `.factory/claims.json` is absent, so there were zero declared claim commands to run. Four Rust unit tests, two Rust integration tests, and four Vitest tests pass, but none is tagged to a public claim and several claims have no equivalent end-to-end test. The 14 untested claim families are listed below. This finding alone prevents a strict PASS.

5. **The first screen and section copy do not meet the plain-words contract.** The first-screen sentence is 31 words and does not directly name the team audience. The screen lacks the required three short facts about privacy, offline use, and price. The title and headline say “every database query” and “the database” although the product supports file-backed SQLite only. Labels such as “specimen no. 001,” “field test,” “receipt notebook,” “method, not middleware,” and “press” use the field-guide metaphor instead of naming the sections and actions directly. `.factory/copy-audit.md` is absent.

6. **The site has no real demo or 404 route.** `/not-a-real-route-qa` returns HTTP 200 and the home page instead of a deliberate HTTP 404 page. `/demo` also returns the home page and keeps the home title rather than `Demo — DB Access Receipts`. There is no 404 document or Static Web Apps response override. The sitemap consequently has no demo or 404 URL.

### Low

7. **Required metadata and shared site structure are incomplete.** The pages have useful titles and descriptions, but no canonical link, Open Graph fields, Twitter card, 1200×630 social image, or apple-touch icon. Footers omit “Built by Param Factory” and a build id. The legal-page headers and footers do not use the same navigation and one-line product summary as the home page.

8. **Required review documents are missing.** `.factory/demo.md` and `.factory/copy-audit.md` do not exist. The visual thesis and asset provenance are present in `.factory/design.md`.

9. **The earlier response-policy finding remains open.** Live responses still have no `Content-Security-Policy`, `Permissions-Policy`, or frame-ancestor protection. HSTS, `Referrer-Policy`, and `X-Content-Type-Options` are present. No exploit was observed.

10. **The earlier cache-policy finding remains open.** Hashed JavaScript and CSS still return `Cache-Control: public, must-revalidate, max-age=30`, not long-lived immutable caching. The service worker provides working offline navigation, so this is a repeat-visit performance issue rather than an offline failure.

11. **Some accessibility requirements fail manual inspection despite clean axe scans.** On a 390 px viewport, visible header/footer/legal links measured 15–40 px high rather than the required 44 px touch target. The required actor and account fields are labelled but the form does not explain that they are required. Focus rings, keyboard activation, reduced motion, contrast scans, semantic structure, and responsive overflow checks passed.

## Untested public claims

The missing claims manifest leaves these 14 claim families untested under the required claim contract. Some have incidental unit or manual evidence, but none has the required single tagged command and isolated sandbox assertion.

1. Named templates enforce declared parameters and limits.
2. Novel SQL requires an attached terminal and one-use human challenge.
3. Non-interactive novel SQL is denied and still receipted.
4. Read-only opening, single-statement enforcement, and write denial prevent database changes.
5. Column caps reject and row caps truncate.
6. Successful, denied, and failed attempts receive signed receipts.
7. Receipts verify offline and tampering is detected.
8. Receipts omit raw SQL, parameter values, credentials, database paths, and result cells.
9. Database URLs and signing keys use the operating-system keychain in normal use.
10. Documented JSON output and exit codes work across success, denial, query failure, and signature failure.
11. The CLI has no telemetry and the browser sends no query or demo data away.
12. The site and walkthrough work offline after the first visit.
13. License restore, cached unlock, daily verification, revocation, and offline fallback work as stated.
14. A valid purchase supplies the listed Team Field Kit download and major-version updates.

The live install command is a false claim and the buy action is a broken public path; they are Findings 1 and 3 rather than additional untested-claim counts.

## Demo and data-safety evidence

Fresh Chromium contexts at 1440×1000 and 390×844 showed the primary action without scrolling. After clicking it and then running the prefilled template, the page displayed **Allowed and signed**, actor `analyst@team`, `2/50 rows`, a six-column cap, a query hash, and an approval path. The receipt history survived reload. **Clear local history** removed the `db-receipts:demo-receipts` entry and restored the empty state.

The browser flow made only same-origin document, script, style, and image requests. No query, actor, account value, or receipt was sent over the network. The only browser state created by the sample flow was the documented local-storage key. This proves that the observed walkthrough did not change real data. It does not cure the missing demo mode and labels.

Browser invalid and recovery paths also worked: an incorrect challenge created a signed denial, `FERN-42` then allowed the read, and a `DELETE` statement was denied and receipted.

## CLI evidence from a clean consumer

`cargo package` produced `db-access-receipts-0.1.0.crate`. I installed the unpacked package into a new temporary Cargo root and exercised that installed binary against a newly created SQLite database using the documented headless environment overrides.

- `--help`, `--json init`, and `--json templates` worked. Re-running initialization is covered by the existing integration evidence.
- The shipped `examples/db-receipts.toml` ran `open-orders` and returned two real rows plus a signed receipt. Offline verification returned `valid:true`.
- Missing parameters and an unknown template exited 2 and wrote denial receipts.
- Sixty matching rows were capped to 50 with `truncated:true`.
- A wrong interactive challenge was denied; retrying with the new challenge succeeded.
- `DELETE FROM orders` remained denied after a valid challenge, and the database still contained all three original rows.
- A modified receipt exited 4 with `receipt signature is invalid`.
- Receipt directories/files were mode 0700/0600. Searches found no raw account value, SQL, database path, or returned timestamp in the receipts.
- `db-receipts demo` exited 2 because the command does not exist.

This confirms the core safety behavior, but the installed artifact cannot be obtained through the command advertised on the live site.

## Build and automated checks

From the clean checkout, all documented local commands passed after `npm ci`:

| Command | Result |
| --- | --- |
| `cargo fmt --all -- --check` | PASS |
| `cargo clippy --all-targets --all-features -- -D warnings` | PASS |
| `npm test` | PASS: 4 Rust unit, 2 Rust integration, 4 Vitest tests |
| `npm run build` | PASS; wrote `dist/site/` |
| `cargo build --release` | PASS |
| `cargo package` | PASS; warned that `tests/cli.rs` is excluded from the package |

`npm ci` reported zero vulnerabilities. The built payload is 8,567 bytes JavaScript and 10,727 bytes CSS, with 22,716-byte and 80,222-byte responsive WebP images.

There were no claim commands to run because `.factory/claims.json` is missing.

## Browser, accessibility, privacy, and performance

- Fresh desktop and phone contexts had one `h1`, one `main`, `lang=en`, no horizontal overflow, 16 px body text, labelled controls, image alt text, and no console or page errors.
- The skip link was the first Tab stop with a 3 px visible focus ring. Tab selection arrows and the theme button with Space worked. No keyboard trap was observed.
- Axe WCAG 2 A/AA scans returned zero violations on `/`, `/demo`, `/?demo=1`, `/privacy/`, `/terms/`, and the fallback URL in light mode. The interactive page also returned zero violations in dark mode.
- Reduced motion changed receipt animation to `0.01ms` and smooth scrolling to `auto`.
- Service-worker-controlled offline reload worked for home, privacy, and terms, with the offline notice shown.
- Invalid license restore called only the documented Sociobot verification endpoint, stayed locked, and showed a useful error. No credential or real license was used.
- `/opt/fleet/lib/verify-url.sh` passed after its required evidence directory was created: HTTP 200, title, `lang`, one `h1`, main landmark, alt text, and no console errors.
- Lighthouse mobile: Performance 100, Accessibility 100, Best Practices 100, SEO 100; LCP 1.20 s, CLS 0, TBT 27 ms.

The site is static and the product is a local CLI. Tenant isolation, backend restart persistence, health endpoints, and product-backend 429/`Retry-After` checks are not applicable. No other product, shared database, staging slot, or unrelated secret was accessed.

## Earlier findings and decisions

| Earlier item | Current disposition |
| --- | --- |
| Hashed assets lack immutable caching | **Open** as Finding 10; live headers are unchanged. |
| CSP, Permissions Policy, and frame protection absent | **Open** as Finding 9; live headers are unchanged. |
| Checkout awaits product registration | **Open and user-visible** as Finding 3; the live buy link returns 404. |
| SQLite-only v0.1 | Accepted scope; copy must state SQLite consistently. The core implementation works. |
| OS keychain depends on host support | Accepted and documented; clean-container overrides worked. Normal keychain behavior has no claim test. |
| No key rotation or organization trust registry | Documented non-goal for v0.1; no contrary public promise found. |
| Browser walkthrough is illustrative, not a SQL engine | Disclosure is accurate. The separate required demo contract still fails as Finding 2. |

The prior report called the product PASS while listing two low-severity findings. This review uses the work order's strict rule: any finding or untested public claim requires FAIL.

## Evidence files

- `/work/.evidence/live-browser.json`
- `/work/.evidence/browser-paths.json`
- `/work/.evidence/desktop-first-screen.png`
- `/work/.evidence/phone-first-screen.png`
- `/work/.evidence/verify-url/verify.json`
- `/work/.evidence/lighthouse.json`

## Required next steps

1. Publish the crate or replace the live install command with a tested available artifact path.
2. Add the CLI demo, real sample entry URL, persistent demo controls, and landing-page recording.
3. Register and verify the checkout before showing the buy action.
4. Add `.factory/claims.json` and one tagged sandbox test for every retained public claim.
5. Replace metaphor copy, state the audience and three facts on the first screen, and add the required copy audit.
6. Add real demo and 404 routes, complete metadata/site structure, response headers, cache rules, and touch targets.
