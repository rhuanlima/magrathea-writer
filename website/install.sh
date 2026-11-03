#!/usr/bin/env bash
# ==============================================================================
# Instalador Rápido & Desbloqueador Gatekeeper — Magrathea Writer (macOS)
# Uso: curl -fsSL https://rhuanlima.github.io/magrathea-writer/install.sh | bash
# ==============================================================================

set -euo pipefail

REPO="rhuanlima/magrathea-writer"
APP_NAME="Magrathea Writer.app"
DEST_APP="/Applications/${APP_NAME}"

echo "🪐 Magrathea Writer — Instalador Oficial para macOS"
echo "----------------------------------------------------"

# Detectar arquitetura do Mac (Apple Silicon arm64 vs Intel x86_64)
ARCH="$(uname -m)"
if [ "$ARCH" = "arm64" ]; then
  ARCH_PATTERN="aarch64"
else
  ARCH_PATTERN="x64"
fi

echo "🔍 Buscando a versão mais recente no GitHub (${REPO})..."
RELEASE_JSON="$(curl -fsSL "https://api.github.com/repos/${REPO}/releases" || true)"

DMG_URL=""
if [ -n "$RELEASE_JSON" ]; then
  DMG_URL="$(printf '%s' "$RELEASE_JSON" | grep -Eo "https://github.com/${REPO}/releases/download/[^\"]+_${ARCH_PATTERN}\\.dmg" | head -n 1 || true)"
  if [ -z "$DMG_URL" ]; then
    DMG_URL="$(printf '%s' "$RELEASE_JSON" | grep -Eo "https://github.com/${REPO}/releases/download/[^\"]+\\.dmg" | head -n 1 || true)"
  fi
fi

if [ -n "$DMG_URL" ]; then
  TMP_DMG="$(mktemp -t magrathea_installer).dmg"
  MOUNT_POINT="$(mktemp -d -t magrathea_mount)"

  cleanup() {
    hdiutil detach "$MOUNT_POINT" -quiet 2>/dev/null || true
    rm -f "$TMP_DMG"
    rmdir "$MOUNT_POINT" 2>/dev/null || true
  }
  trap cleanup EXIT

  echo "⬇️  Baixando pacote DMG sem quarentena de navegador..."
  echo "   URL: ${DMG_URL}"
  curl -fL --progress-bar "$DMG_URL" -o "$TMP_DMG"

  echo "💿 Montando imagem de disco..."
  hdiutil attach "$TMP_DMG" -mountpoint "$MOUNT_POINT" -nobrowse -quiet

  echo "📂 Instalando '${APP_NAME}' em /Applications..."
  rm -rf "$DEST_APP"
  cp -R "${MOUNT_POINT}/${APP_NAME}" "/Applications/"
elif [ -d "$DEST_APP" ]; then
  echo "ℹ️  Usando instalação existente em ${DEST_APP}..."
else
  echo "❌ Não foi possível localizar o pacote DMG para download."
  exit 1
fi

echo "🔓 Removendo atributos de quarentena (xattr) e selando assinatura ad-hoc..."
xattr -cr "$DEST_APP" 2>/dev/null || true
codesign --force --deep -s - "$DEST_APP" 2>/dev/null || true

echo "✅ Magrathea Writer instalado e liberado com sucesso!"
echo "🚀 Abrindo o aplicativo..."
open -a "$DEST_APP"
