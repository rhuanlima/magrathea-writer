# Magrathea Writer

🇬🇧 [English](README.md) · 🇧🇷 Português

*Sua oficina particular de criação de mundos.*

Ambiente de escrita e "segundo cérebro" literário, **local-first, offline e cifrado**, para autores de ficção e worldbuilders. Manuscrito, fichas de mundo e linha do tempo ficam num **cofre cifrado** na sua máquina: nenhum outro aplicativo lê seus arquivos. Quando quiser sair ou compartilhar, **exporte** (Markdown aberto para Obsidian, PDF, DOCX, EPUB).

> No *Guia do Mochileiro das Galáxias* (Douglas Adams), Magrathea é o planeta que fabrica planetas sob encomenda, com montanhas e litorais esculpidos à mão. Este app é a oficina para o seu.

**Licença:** [GNU AGPL v3](LICENSE) — software livre; derivados e uso em rede devem continuar AGPL, com atribuição ao projeto original. **Sem garantia de qualquer tipo: faça backup do seu cofre e guarde sua chave de recuperação em local seguro** — a perda conjunta da senha e da chave de recuperação torna o conteúdo irrecuperável por princípio de segurança.

**Repositório:** https://github.com/rhuanlima/magrathea-writer

---

## Tecnologias

- **Core & Shell:** [Tauri 2](https://v2.tauri.app/) (Rust) — janela nativa offline leve, criptografia com segurança de memória e armazenamento local em cofre.
- **Interface Frontend:** [Svelte 5](https://svelte.dev/) + [TypeScript](https://www.typescriptlang.org/) + [Vite](https://vitejs.dev/) — interface reativa com excelente desempenho sem sobrecarga de DOM virtual.
- **Editor Literário:** [CodeMirror 6](https://codemirror.net/) — escrita livre de distrações com foco dinâmico e suporte a Markdown.

---

## Como Executar

### Pré-requisitos

- [Node.js](https://nodejs.org/) (versão 20+ recomendada) e `npm`
- Toolchain do [Rust](https://www.rust-lang.org/) (`rustc` e `cargo`)

### Instalação e Desenvolvimento

```bash
# Instalar dependências de frontend e do Rust
npm install
cargo check --manifest-path src-tauri/Cargo.toml

# Executar a aplicação desktop em modo de desenvolvimento
npm run tauri dev
```

### Qualidade e Testes

```bash
# Checagem de formatação e lint (ESLint, Cargo Clippy, Cargo fmt)
npm run lint
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check

# Checagem de tipos Svelte e TypeScript
npm run typecheck

# Execução das suítes de testes (Vitest e Cargo test)
npm test
cargo test --manifest-path src-tauri/Cargo.toml

# Compilação para produção
npm run build
npm run tauri build
```

---

## Contribuição

Contribuições exigem DCO sign-off (`git commit -s`) do colaborador humano. Todo o desenvolvimento adota fluxo baseado em branches e Pull Requests com merge via squash.
