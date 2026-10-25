# Magrathea Writer

🇬🇧 [English](README.md) · 🇧🇷 Português

*Sua oficina particular de criação de mundos.*

Ambiente de escrita e "segundo cérebro" literário, **local-first, offline e cifrado**, para autores de ficção e worldbuilders. Manuscrito, fichas de mundo e linha do tempo ficam num **cofre cifrado** na sua máquina: nenhum outro app lê seus arquivos. Quando quiser sair ou compartilhar, **exporte** (Markdown aberto para Obsidian, PDF, DOCX, EPUB).

**Licença:** [GNU AGPL v3](LICENSE) — software livre; derivados e uso em rede devem continuar AGPL, com atribuição ao projeto original. Sem garantia de qualquer tipo; **faça backup e guarde sua chave de recuperação** (ver SECURITY.md). Apoie o projeto com um café ☕ (seção no app).

> No *Guia do Mochileiro das Galáxias* (Douglas Adams), Magrathea é o planeta que fabrica planetas sob encomenda, com montanhas e litorais esculpidos à mão. Este app é a oficina para o seu.

**Repositório:** https://github.com/rhuanlima/magrathea-writer

## Documentos

| Arquivo | Para quê |
|---|---|
| [PRD.md](PRD.md) | O que construir e por quê (escopo da v1, histórias, decisões) |
| [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) | Como construir (Tauri 2 + TypeScript + CodeMirror 6, cofre cifrado) |
| [DESIGN.md](DESIGN.md) | Sistema visual (tokens, tipografia, componentes, modo escuro) |
| [SECURITY.md](SECURITY.md) | Modelo de ameaça e regras de segurança |
| [AGENTS.md](AGENTS.md) | Regras para agentes de IA (branches, PRs, sem assinatura, backlog); `GEMINI.md` aponta para ele |
| [todo.md](todo.md) | Backlog único, por marcos |
| `references/` | Telas-modelo (HTML + PNG) |
| `assets/brand/` | Conceito de logo e ícone |
| `docs/archive/` | Versões antigas (PRD v1/v2, arquitetura Flutter, design system mono) — só histórico |

## Como começar a desenvolver

Leia [AGENTS.md](AGENTS.md), pegue o primeiro item de **Próximos** no [todo.md](todo.md), crie uma branch e abra um PR.
Pendências do autor antes de começar: ver **Decisões pendentes** no `todo.md`.
