# Website Oficial — Magrathea Writer

Este diretório contém a **Landing Page oficial** do Magrathea Writer, projetada para ser 100% estática e pronta para publicação direta no **GitHub Pages**.

---

## 📁 Estrutura de Arquivos

- **[`index.html`](index.html)**: Landing page completa, responsiva e alinhada às diretrizes visuais do *Atelier Scriptorium* (`docs/DESIGN.md`).
  - Hero com proposta de valor, badges de garantia e links de download.
  - Vitrine interativa simulando o aplicativo (Editor Zen, Árvore do Manuscrito, Segundo Cérebro e Worldbuilding).
  - O Manifesto da Obra Sagrada e a metáfora de Douglas Adams.
  - Especificações de segurança (XChaCha20-Poly1305 + Argon2id).
  - FAQ interativo em sanfona.
  - Seção de apoio com QR Code Pix oficial.
- **`assets/`**:
  - `manifesto-art.jpg`: Ilustração conceitual de alta qualidade retratando o atelier literário e o mundo esculpido.
  - `app-icon.png`, `world-file-light.png`, `world-file-dark.png`, `pix-qrcode.png`.
- **`.nojekyll`**: Garante que o GitHub Pages sirva os arquivos brutos sem processamento do Jekyll.

---

## 🚀 Como Visualizar Localmente

Como a página é totalmente estática (HTML/CSS/JS puros), basta dar um duplo clique no arquivo [`index.html`](index.html) ou abri-lo diretamente em qualquer navegador web (Chrome, Safari, Firefox, Edge).

---

## 🌐 Publicação no GitHub Pages

O projeto já inclui o workflow automático em [`.github/workflows/pages.yml`](../.github/workflows/pages.yml).

### Como ativar em 3 passos:
1. No seu repositório no GitHub, vá em **Settings** > **Pages** (no menu lateral).
2. Na seção **Build and deployment** > **Source**, selecione **GitHub Actions**.
3. Faça push para a branch `main` (ou acione o workflow na aba *Actions*).

A Landing Page estará disponível publicamente em:
`https://<seu-usuario>.github.io/magrathea-writer/`
