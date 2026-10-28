# Contribuindo com o Magrathea Writer

Obrigado pelo seu interesse em contribuir com o **Magrathea Writer**! O Magrathea Writer é uma oficina literária desktop privativa, cifrada, livre de distrações e 100% offline para romancistas e criadores de mundos, distribuída sob a licença **GNU Affero General Public License v3 (AGPLv3)**.

---

## 1. Princípios e Limites Inegociáveis

Antes de escrever código ou abrir um pull request, familiarize-se com nossas diretrizes rígidas:

1. **Offline Absoluto**: Nenhuma conexão de rede em tempo de execução. Zero telemetria, zero requisições a CDNs/fontes externas, zero relatórios de erro remotos ou analytics.
2. **Cifragem Total em Repouso**: A obra do autor é sagrada e confidencial. Todos os arquivos de cenas, capítulos e fichas são cifrados em repouso com **XChaCha20-Poly1305** e derivação de chaves via **Argon2id**. Arquivos temporários em texto claro são terminantemente proibidos.
3. **Liberdade Copyleft (AGPLv3)**: Todo código contribuído é submetido sob a licença AGPLv3.
4. **Certificado de Origem do Desenvolvedor (DCO)**: Contribuidores humanos devem certificar que têm o direito de submeter a contribuição incluindo a linha `Signed-off-by: Nome <email>` em seus commits.

---

## 2. Ambiente de Desenvolvimento

O Magrathea Writer utiliza **Tauri v2**, **Svelte 5** (Runes), **CodeMirror 6** e **Rust**.

### Pré-requisitos
- Node.js 20+ e `npm`
- Rust estável (`rustup update stable`)
- Dependências de build do SO para o Tauri 2

### Comandos de Trabalho
```bash
# Instalação de dependências
npm install
cargo check --manifest-path src-tauri/Cargo.toml

# Execução em modo de desenvolvimento
npm run tauri dev

# Verificação de lint e formatação
npm run lint
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check

# Execução de testes automatizados
npm test
cargo test --manifest-path src-tauri/Cargo.toml

# Compilação de pacote de produção
npm run tauri build
```

---

## 3. Diretrizes para Pull Requests

1. **Um assunto por PR**: Mantenha as alterações focadas em um único item ou correção.
2. **Conventional Commits**: Utilize títulos padronizados como `feat(editor): adiciona modo foco` ou `fix(vault): previne condicao de corrida no salvamento`.
3. **Testes Obrigatórios**: Todo novo comportamento ou correção deve conter testes unitários correspondentes (`vitest` ou `cargo test`).
4. **CI Obrigatório**: Os PRs devem passar com 100% de sucesso nas verificações de integração contínua (lint, testes, auditoria de dependências).
