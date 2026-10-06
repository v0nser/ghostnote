# Coda site assets

First-pass doodles are inline SVG in `src/components/doodles.tsx`. They are layered so a later hand-ink pass can swap groups without rewriting the page.

Replace by keeping the same `viewBox`, `aria-label`, and group names. Prefer SVG over PNG so the wobble filter and idle motion still work.

| Asset | File / group | Layers to keep | Notes |
| --- | --- | --- | --- |
| The Engineer | `Engineer` | body, hair, eyes, mouth, laptop | Tired, hoodie, messy hair. Used in Confusion + Nirvana. |
| The Friend | `Friend` | body, hair, eyes, mouth | Sits beside the Engineer. Confusion, Arrival, Nirvana. |
| Coda Baba | `CodaBaba` | orb, body, hoodie, glasses, scarf, laptop | Calm sage. Orb fill `#e07a3d`. Poses: `stand`, `sit`, `rest`. |
| Chaos marks | `ChaosMarks` | wires, tabs, sticky notes | Confusion panel only. Respect `prefers-reduced-motion` (no jitter). |
| Lotus | `Lotus` | petals | Nirvana halo. Decorative; `aria-hidden`. |
| Ink wobble | `InkFilter` (`#ink-wobble`) | turbulence + displacement | Applied via `.wobble`. Do not apply to body copy. |
| Favicon | `public/favicon.svg` | Baba silhouette + orb | Tiny version of Baba. |
| Paper grain | `src/styles/global.css` body background | feTurbulence data-URI | Warm off-white `#f3ead8`. |

Motion that should survive an illustration swap:

- `.breathe` — Nirvana scene, ~2% scale / 4s
- `.jitter` — chaos marks, mouse-independent idle (disabled when reduced motion)
- GSAP ScrollTrigger — comic panels on desktop only

Copy is not an asset; it lives in `src/content/site.ts`.
