# Independent verification — PASS

**Candidate:** `7bbfcf546b4b97801dd9b99c16d5e81e0c7df879`  
**Live URL:** <https://db-access-receipts.sociobot.in/>  
**Verified:** 2026-08-27/28 UTC from a clean working tree at the candidate commit.

## Verdict

**PASS.** The candidate meets the researched brief's smallest useful product: the shipped Rust CLI locally permits bounded named read queries, requires an interactive human challenge for novel SQL, refuses writes even after approval, and writes verifiable data-minimizing Ed25519 receipts. The production deployment is live and byte-for-byte matches the candidate's built site for the home page, JS, CSS, service worker, privacy page, and terms page.

Two low-severity deployment hardening findings are recorded below; neither changes the demonstrated safety boundary or blocks this candidate.

## Local clean-build evidence

`npm ci` completed with 0 vulnerabilities reported. The following all passed:

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo build --release
cargo package --allow-dirty
npm test
npm run build
```

Results: 4 Rust library tests, 2 Rust binary integration tests, 4 Vitest tests, release build, package verification, TypeScript checking, and Vite production build all passed. `cargo package` produced and verified `db-access-receipts v0.1.0` (103.5 KiB unpacked / 28.1 KiB compressed, 10 files). `npm run build` produced `dist/site/` with 8,567-byte JS and 10,727-byte CSS.

The packed crate was installed from `target/package/db-access-receipts-0.1.0` into a new temporary consumer root with `cargo install --path ... --root ... --locked`. Its installed `db-receipts --help`, `--json init`, and `--json templates` worked; a second `init` correctly returned exit code 2 rather than overwriting the policy.

## Independent CLI exercises

Using a freshly created SQLite database, fresh policy, and the documented explicit CI signing/database overrides:

- Listed the allowlisted template and ran it with `account_id=acct_123`; received two bounded rows and a signed receipt. Offline `verify` returned `valid: true`.
- In a real TTY, supplied the one-use randomized challenge for a novel parameterized `SELECT`; it succeeded. A three-row query under a two-row cap returned two rows with `truncated: true`.
- A three-column query under a two-column cap was denied after the human challenge, with a signed denial receipt.
- `DELETE FROM orders` was denied after a valid human challenge (exit 2, `write or schema-changing SQL is never allowed`); no write passed.
- Invalid supplied parameters and an unknown template were denied (exit 2) and receipted. A novel request without a TTY was denied and receipted.
- A deliberately modified receipt failed offline verification with exit 4. Receipt directory/file modes were `0700`/`0600`; receipt searches found neither the raw account value, raw SQL, nor database path.

## Browser, accessibility, privacy, and PWA evidence

Ran `/opt/fleet/lib/verify-url.sh` against both the locally built site and the live URL. Both returned HTTP 200, title, `lang=en`, exactly one `h1`, a `main` landmark, all image alt attributes, and no console/page errors.

Playwright 1.58.2 plus axe independently exercised desktop (1440×1000) and mobile (390×844) on both artifacts:

- Axe: 0 violations, including 0 serious/critical findings.
- Template allow, novel-write denial, incorrect-challenge denial/recovery, and correct novel approval all produced their stated receipt states.
- Arrow-key tab selection works; focused controls have a visible 3px pollen-colour ring; the theme control works with Space.
- At 390 px, there was no document horizontal overflow and body text was 16 px.
- With reduced motion, receipt animation duration became `0.01ms` and `scroll-behavior` became `auto`.
- No initial outbound request was made except same-origin document/assets/images. No analytics, third-party fonts, CDN scripts, or query/demo-data requests were observed. The only coded external data call is the documented Sociobot license verification after a license is supplied.
- After service-worker activation and one controlled reload, an offline reload succeeded and rendered the cached home `h1`; the page had a service-worker controller. The worker uses `skipWaiting`/`clients.claim`, clears old named caches, and serves cached shell navigation offline.

Mobile Lighthouse against the live deployment: Performance **99**, Accessibility **100**, Best Practices **100**, SEO **100**; LCP **1.219 s**, CLS **0**, TBT **104 ms**. The production initial JS (8.6 KB) and CSS (10.7 KB) are under budget; the responsive mobile hero is 22.7 KB and the large hero is 80.2 KB, both under the 300 KB budget.

## Deployment and response evidence

Live HTTP/2 home, JS, CSS, and service-worker responses were 200 with HTTPS/HSTS, `Referrer-Policy: strict-origin-when-cross-origin`, `X-Content-Type-Options: nosniff`, and no browser errors. SHA-256 comparisons prove the live deployment is this candidate:

| Artifact | SHA-256 |
| --- | --- |
| `/` and `dist/site/index.html` | `8237f250804a6a9c1e65391fe620c47273a8516625d1d11200550ec2eb4c7f7e` |
| `/assets/styles-BGQhTGFT.css` | `e00b5d9827205df62d701f32e9cd41cbdab2383a468be20201fed0506d46fa83` |
| `/assets/index-Dg6ZenrJ.js` | `275724f6511f8f43f829d4ec8ec44ac15f2b9f7b425b8c491ec8193954d4099a` |
| `/service-worker.js` | `c7ff94196232078e0287bb21fe0ad1ffad8642833e32ceaf5596bf097d132047` |
| `/privacy/` | `dc7ed6968efb65790452916a79ae267f18d564c87e429a00f3263e3cca3fb6e2` |
| `/terms/` | `0f9f33b5fce0f488044afa20f46104e8c4b3fdb54183fe6b005c886a599924c6` |

## Defects by severity

### High / medium

None.

### Low

1. **Static assets are not immutable-cached.** The live hashed JS/CSS and service worker use `Cache-Control: public, must-revalidate, max-age=30`, rather than long-lived `immutable` caching. This costs repeat-visit performance and does not meet the performance skill's recommended static-asset cache policy. The service worker provides offline caching, so this is not a functional or first-load budget failure.
2. **Defence-in-depth response policies are absent.** The live responses have no `Content-Security-Policy`, `Permissions-Policy`, or frame-ancestors protection (`X-Frame-Options` is also absent). HSTS, referrer policy, and `nosniff` are present. A restrictive CSP is especially advisable because an optional license token is stored locally. This is a deployment/header configuration follow-up; no XSS or data exfiltration path was found in the candidate.

## Reproduce

```sh
npm ci
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
npm test
npm run build
cargo build --release
cargo package
```

Then serve `dist/site/` and run the supplied `/opt/fleet/lib/verify-url.sh`, axe, and Lighthouse checks as recorded above. Do not publish from this checkout; the factory owns registry/deployment credentials.
