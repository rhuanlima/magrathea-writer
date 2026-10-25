# Magrathea Writer

🇬🇧 English · 🇧🇷 [Português](README.pt-BR.md)

*Your private workshop for building worlds.*

A **local-first, offline and encrypted** writing environment and literary "second brain" for fiction authors and worldbuilders. Your manuscript, world notes and timeline live in an **encrypted vault** on your machine: no other app can read your files. When you want to leave or share, **export** (open Markdown for Obsidian and friends, PDF, DOCX, EPUB).

> In Douglas Adams' *The Hitchhiker's Guide to the Galaxy*, Magrathea is the planet that builds custom planets, with hand-sculpted mountains and coastlines. This app is the workshop for yours.

**License:** [GNU AGPL v3](LICENSE) — free software; derivatives and network use must remain AGPL, crediting the original project. **No warranty of any kind: back up your vault and keep your recovery key safe** — losing both your password and recovery key means your work is unrecoverable by design (see [SECURITY.md](SECURITY.md) §7).

**Repository:** https://github.com/rhuanlima/magrathea-writer

## Documents

Internal planning documents (PRD, backlog, security model, architecture) are written in Brazilian Portuguese; code, commits, PRs and public-facing docs are in English.

| File | Purpose |
|---|---|
| [PRD.md](PRD.md) | What to build and why (v1 scope, user stories, decisions) |
| [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) | How to build it (Tauri 2 + TypeScript + Svelte + CodeMirror 6, encrypted vault) |
| [DESIGN.md](DESIGN.md) | Visual system (tokens, typography, components, dark mode) |
| [SECURITY.md](SECURITY.md) | Threat model, cryptography requirements, liability |
| [AGENTS.md](AGENTS.md) | Rules for AI agents (branches, PRs, no signing, backlog); `GEMINI.md` points to it |
| [todo.md](todo.md) | The single backlog, by milestone |
| `references/` | Visual mock-ups (HTML + PNG) |
| `assets/brand/` | Logo and icon concept |
| `docs/archive/` | Superseded drafts, history only |

## Contributing

Read [AGENTS.md](AGENTS.md), pick the first item under **Próximos** in [todo.md](todo.md), create a branch and open a PR. Contributions require a DCO sign-off (`git commit -s`) by the human contributor. `CONTRIBUTING.md` (and `CONTRIBUTING.pt-BR.md`) will be published before the first release.
