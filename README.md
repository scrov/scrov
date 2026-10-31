# scrov

[![CI](https://github.com/scrov/scrov/actions/workflows/ci.yml/badge.svg)](https://github.com/scrov/scrov/actions/workflows/ci.yml)
[![Sync Projects](https://github.com/scrov/scrov/actions/workflows/sync.yml/badge.svg)](https://github.com/scrov/scrov/actions/workflows/sync.yml)
[![Deploy](https://github.com/scrov/scrov/actions/workflows/deploy.yml/badge.svg)](https://github.com/scrov/scrov/actions/workflows/deploy.yml)
[![Website](https://img.shields.io/badge/website-scrov.app-blue)](https://scrov.app)

A terminal portfolio, blog, and developer hub built with Astro, Tailwind CSS, Bun, and Cloudflare.

Live demo: **[https://scrov.app](https://scrov.app)**

## Features

- **Distro & Terminal Theming**: Switch between 20+ Linux and BSD distribution aesthetic palettes (Debian, Arch, Ubuntu, Kali, Fedora, Void, NixOS, etc.) with zero flicker.
- **YAML-Driven Architecture**: All data, projects, system configurations, and bio details live in `src/data.yml` and `src/systems.yml`.
- **GitHub Sync Automation**: Automatically fetches live repo stats (stars, descriptions, sizes, update dates) via GitHub Actions and caches locally with configurable TTL.
- **Strict Type Checking & Linting**: Powered by TypeScript strict mode and Biome.
- **Cloudflare Edge Deployment**: Serverless edge SSR and static assets powered by Cloudflare Workers and `@astrojs/cloudflare`.

## Tech Stack

- **Framework**: [Astro 5](https://astro.build)
- **Styling**: [Tailwind CSS 4](https://tailwindcss.com) via `@tailwindcss/vite`
- **Runtime & Package Manager**: [Bun](https://bun.sh)
- **Adapter**: `@astrojs/cloudflare`
- **Linter & Formatter**: [Biome](https://biomejs.dev)
- **CI/CD**: GitHub Actions

## Project Structure

```
.
├── .github/
│   ├── ISSUE_TEMPLATE/       # GitHub issue templates (bug reports, feature requests)
│   ├── workflows/
│   │   ├── ci.yml            # CI validation: build + tsc + bun test
│   │   ├── sync.yml          # Scheduled & triggered GitHub project sync
│   │   └── deploy.yml        # Cloudflare deployment via Wrangler
│   ├── dependabot.yml        # Dependabot configuration (npm + actions)
│   ├── FUNDING.yml           # GitHub Sponsors configuration
│   └── pull_request_template.md
├── public/                   # Static assets, fonts, icons
├── scripts/
│   ├── lib/project.ts        # GitHub API helpers and formatting
│   ├── sync.ts               # YAML + GitHub -> src/config.json generator
│   └── sync.test.ts          # Bun unit tests for sync logic
├── src/
│   ├── components/           # UI and TUI components (btop, lazygit, mutt, ranger)
│   ├── layouts/              # Astro layouts
│   ├── pages/                # Astro pages & API routes
│   ├── data.yml              # Profile, host, and contact data
│   ├── systems.yml           # Distro theme definitions
│   └── config.json           # Generated configuration
├── astro.config.mjs          # Astro + Tailwind + Cloudflare configuration
├── package.json              # Scripts and dependencies
├── tsconfig.json             # TypeScript configuration
└── wrangler.json             # Cloudflare Workers configuration
```

## Getting Started

### Prerequisites

- [Bun](https://bun.sh) `>= 1.1` (or Node.js `>= 22.12.0`)
- Optional: GitHub Personal Access Token (`GH_TOKEN` / `GITHUB_TOKEN`) to avoid unauthenticated API rate limits when syncing projects.

### Installation

```bash
bun install
```

### Development

```bash
bun run dev
```

### Sync GitHub Projects

Refreshes project metadata in `src/config.json`:

```bash
bun run sync
```

### Run Tests and Typecheck

```bash
# Run all unit tests
bun run test

# Run full project validation (build + typecheck + tests)
bun run check
```

### Production Build

```bash
bun run build
```

### Deployment

Deploy to Cloudflare Workers:

```bash
bun run deploy
```

## Contributing

Pull requests and issues are welcome! Please make sure that `bun run check` passes before submitting a pull request.
