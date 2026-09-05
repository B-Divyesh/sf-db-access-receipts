# Visual thesis — the query herbarium

## Direction and rationale

DB Access Receipts is a botanical field guide for database access. A field guide does not control the landscape; it helps a careful observer identify a specimen, record its boundaries, and leave a durable note. That is exactly the product's relationship to SQL: each query is classified, bounded, and pressed into a verifiable receipt. The interface uses specimen labels, ruled annotations, accession numbers, and a single original “query fern” instead of generic security shields or gradient-heavy SaaS chrome.

## Palette

The light treatment is explicitly a warm archival-paper field guide. The dark treatment becomes a night field notebook. All combinations used for text meet WCAG AA.

| Token | Light | Dark | Use |
| --- | --- | --- | --- |
| background | `#F3EFE3` | `#101813` | uncoated paper / field notebook |
| surface | `#FCFAF3` | `#17231C` | specimen sheets |
| text | `#17231C` | `#F2ECDD` | carbon ink |
| muted text | `#4D554F` | `#B6BDAF` | graphite annotations |
| accent | `#1F684B` | `#76C69C` | fern green; primary action |
| accent contrast | `#FFFFFF` | `#0E2017` | button labels |
| pollen | `#D1A43C` | `#E4BF64` | accession markers |
| success | `#1B613D` | `#72C797` | verified states |
| warning | `#8A5908` | `#F0C66C` | bounded/truncated states |
| danger | `#9B3E2C` | `#F28F77` | denied/invalid states |
| rule | `#C9C2AD` | `#3B4B40` | quiet dividers |

## Type and spacing

Headings use the local serif stack `Georgia, Cambria, Times New Roman, serif`, giving the specimen-sheet voice without a network font. Controls, code, labels, and numbers use `ui-monospace, SFMono-Regular, Menlo, Consolas, monospace` for exactness and tabular figures. Body copy uses the system humanist sans stack for long-form clarity. No font files or runtime font requests are required.

The scale is 16, 18, 23, 31, 44, and 64 px. Body line-height is 1.6 and measures stay below 70 characters. Spacing follows a 4/8 px rhythm: 4, 8, 12, 16, 24, 32, 48, 64, and 96 px. Controls are at least 44 px high.

## Layout and interaction grammar

- A thin accession rail and ruled dividers establish hierarchy before boxes do.
- Independent artifacts (policy, query, receipt) are specimen sheets with clipped corners; explanatory text is unboxed.
- States always have a word and icon/shape, never color alone.
- Primary actions resemble dark-green catalogue stamps. Pressing translates them by one pixel; focus uses a 3 px pollen ring.
- The interactive demo moves left-to-right from query identification to receipt. On phones it stacks and omits secondary annotations, never the approval state.

## Motion policy

Motion is restrained and physical: the receipt enters 12 px from its source over 220 ms, while state changes cross-fade over 160 ms. The hero specimen has no looping motion. With `prefers-reduced-motion: reduce`, transforms and smooth scrolling are removed and changes are immediate opacity swaps. No flashing is used.

## Original asset plan and provenance

`site/public/query-fern.webp` and its 640 px responsive derivative are the hero specimen: a pressed fern whose leaflets subtly resolve into orderly database rows, accompanied by a blank accession tag and a small red approval thread. It explains the product's “classify, bound, record” model while leaving page copy to semantic HTML. Both WebP files were stripped and compressed locally (79 KB and 23 KB).

- Generator: `/opt/fleet/lib/gen-image.sh`, factory `factory-image` deployment.
- Prompt: “Use case: illustration-story. Asset type: landing page hero specimen. Primary request: an original botanical field-guide plate showing one pressed fern whose leaflets subtly transition into tidy rows of tiny abstract database cells, beside a blank archival accession tag secured with a short rust-red approval thread. Scene/backdrop: warm uncoated ivory herbarium paper with faint graphite measurement rules and subtle natural fibers. Style/medium: refined hand-painted gouache and colored-pencil scientific illustration, restrained editorial detail, tactile and credible rather than whimsical. Composition/framing: landscape, specimen centered slightly right with generous calm negative space, isolated plate, no border. Color palette: deep fern green, moss, carbon ink, muted pollen gold, tiny rust accent. Lighting/mood: flat museum scan, careful and trustworthy. Constraints: no words, no letters, no numbers, no logos, no watermark, no people, no gradients, no photorealistic UI, no shield or lock icons.”
- License: generated specifically for this repository; released with the project under MIT.

`site/public/sf-db-access-receipts-social.webp` is a 1200×630 crop composed locally from the same generated query-fern artwork for social cards. `site/public/sf-db-access-receipts-apple-touch.png` is a 180×180 crop from its responsive derivative. `site/public/cli-demo.svg` is hand-authored from the real `db-receipts demo` terminal output; it has a visible HTML transcript alongside it. These are original product assets and introduce no external runtime requests.
