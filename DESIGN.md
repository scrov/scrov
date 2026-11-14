# DESIGN.md

**Scope:** the scrov product pages: `/platform/` and `/lab/intent/`. The portfolio's 26 distro themes stay in `src/systems.yml` and are separate from this.

Direction: **terminal-precise signal flow.** It extends scrov.app's arch palette: a near-black ground, monospace labels, and one blue for action. scrov sends one voice request to many devices, so the page shows that as a signal path, not as decoration. Dark is the default. Light mode follows `prefers-color-scheme`.

This direction comes from the scrov brand (`src/systems.yml`, arch palette, `theme_color #1793d1`). It was not compared against reference sites.

Tokens live in `public/platform/tokens.css`. Pages use the variable names, never raw values.

## Color

| Token | Dark | Light | Role |
|---|---|---|---|
| `--bg` | `#08090b` | `#f7f8fa` | Page background |
| `--surface` | `#0e1014` | `#ffffff` | Cards, panels, nav |
| `--surface-2` | `#14171c` | `#eef0f3` | Code blocks, nested panels, hover fill |
| `--border` | `#23272e` | `#dde1e6` | Card and divider lines |
| `--border-strong` | `#343a44` | `#c4cad2` | Inputs, secondary buttons, chips |
| `--text` | `#e6e8eb` | `#0e1116` | Headings, body |
| `--text-muted` | `#a0a6b0` | `#4a515c` | Secondary copy |
| `--text-subtle` | `#8a909c` | `#5e6672` | Labels, meta (still ≥ 4.5:1) |
| `--primary` | `#1793d1` | `#0b6fa4` | Primary buttons, brand mark |
| `--primary-hover` | `#3aa6de` | `#095d8a` | Primary button hover |
| `--on-primary` | `#08090b` | `#ffffff` | Text on primary (5.7:1 / 5.5:1) |
| `--link` | `#7eb8d8` | `#0b6fa4` | Inline links, focus ring |
| `--accent` | `#e0a050` | `#985a0c` | Only the voice input: mic icon, step 01 |
| `--success` | `#4fb58a` | `#1f7a52` | Device acknowledged (reserved) |
| `--warning` | `#f5c451` | `#8a6100` | Degraded / retrying (reserved) |
| `--error` | `#f26d6d` | `#b42323` | Failed / offline (reserved) |

Rules: one primary and one accent, nothing else. No gradients, glass or glow. A status always pairs its color with a word. "Planned" is a hollow ring plus the word, never a color alone.

## Typography

Fonts are self-hosted in `public/platform/fonts/`, with their OFL license files.

- **JetBrains Mono** (variable): headings, labels, code and nav. It ties the page to scrov.app.
- **Inter** (variable): body copy and buttons.

| Token | Size / line height | Use |
|---|---|---|
| `--fs-label` | 13 / 18, mono 500, +0.06em, uppercase | Eyebrows, column heads |
| `--fs-sm` | 15 / 24 | Captions, notes |
| `--fs-body` | 17 / 28 | Body |
| `--fs-lead` | 20 / 30 | Hero sub-copy |
| `--fs-h3` | 22 / 30, mono 700 | Step titles |
| `--fs-h2` | clamp(28px, 4vw, 40px) / 1.2, mono 700 | Section titles |
| `--fs-h1` | clamp(36px, 5.2vw, 56px) / 1.08, mono 700 | Hero only |

Prose is at most `68ch` wide.

## Spacing

The scale is based on 4: `--s-1` 4 · `--s-2` 8 · `--s-3` 12 · `--s-4` 16 · `--s-5` 24 · `--s-6` 32 · `--s-7` 48 · `--s-8` 64 · `--s-9` 96 · `--s-10` 128 (px).

- Sections: `--section` (96px, 64px below 720px), separated by a 1px `--border` line.
- Content: max width `1200px`, side gutter `--gutter` (24px, 16px on phones).

## Shape and depth

- Radius: `--r-sm` 6 (code, chips in tables), `--r-md` 10 (buttons, calls), `--r-lg` 16 (panels, table, CTA band), `--r-pill` (status chips).
- Depth comes from borders. Dark mode has no shadows. Light mode has one, `--shadow-1`, on panels only.

## Components

- **Buttons**: at least 44px tall, `--r-md`, Inter 600 15px.
  - Primary: `--primary` with `--on-primary`.
  - Secondary: `--border-strong` outline.
  - Quiet: text only in `--link`.
  - Disabled: 45% opacity.
  - Focus (all): `2px solid var(--link)`, offset 2px.
- **Icon button**: 44×44, `aria-label` required.
- **Command block**: `--surface-2`, a `$` prompt, mono 14, scrolls sideways, with a copy button that is shown only when JS runs.
- **Fan-out illustration**: the utterance (accent mic), a dashed rail, the tool calls (mono name, target, args), then the spoken reply. It is always labelled "Illustration" until the product works.
- **Steps**: numbered `01`–`04`, joined by a 1px rail (horizontal on desktop, vertical on phones). Step 01 uses the accent.
- **Device table**: real `<table>` with row headers. Below 760px, each row becomes a stack of labelled lines.
- **FAQ**: `<details>` / `<summary>` with a plus icon that rotates 45° (no motion with reduced motion).
- **Nav**: sticky header. Below 880px, the links fold behind a Menu button (`aria-expanded`). Without JS, the links stay visible.
- **Footer**: mono, in scrov.app's footer voice, plus a no-affiliation line.

## Imagery and icons

- Icons: inline SVG in the Lucide style, 1.75px stroke, `currentColor`, 20px. Decorative icons get `aria-hidden="true"`. No emoji.
- No stock photos or AI art. Visuals are diagrams and code drawn in HTML and CSS.
- Third-party products are named in text only, with no logos.
