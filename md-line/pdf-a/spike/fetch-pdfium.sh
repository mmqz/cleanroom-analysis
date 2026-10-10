#!/usr/bin/env bash
# pdfium 取件脚本（PDF-a spike；PDF-b 起随 mdc-pdf 基建复用）
# 纪律：版本 pin + SHA256 验证 + NOTICES 流程（BSD-3 声明随包）
# 来源：bblanchon/pdfium-binaries（Chromium PDFium 官方树的社区构建轨）
set -euo pipefail

# ---- 版本 pin（升级 = 改此处 + 重跑本脚本 + 复验三探针 + NOTICES 更新）----
PDFIUM_RELEASE="chromium/8086"   # 2026-10 轨道（spike 实测版；non-V8 构建）
ARTIFACT="pdfium-linux-x64"   # non-V8（无 JS 引擎——安全策略 §JS 面：v8 变体禁取
SHA256_EXPECTED="588577cf52dabc1a444988bac841920df54cc2f141801424de97ab04f4fbb935"  # chromium/8086 linux-x64 non-V8

BASE="https://github.com/bblanchon/pdfium-binaries/releases/download"
URL="${BASE}/${PDFIUM_RELEASE}/${ARTIFACT}.tgz"
OUT="bin/pdfium-${PDFIUM_RELEASE#chromium/}-linux-x64.tgz"

mkdir -p bin
echo "[fetch] ${URL}"
curl -fL "${URL}" -o "${OUT}"

echo "[verify] sha256 ..."
SHA256_GOT="$(sha256sum "${OUT}" | cut -d' ' -f1)"
if [ "${SHA256_EXPECTED}" != "PLACEHOLDER_FILL_AFTER_DOWNLOAD" ]; then
  if [ "${SHA256_GOT}" != "${SHA256_EXPECTED}" ]; then
    echo "FAIL: sha256 mismatch
  expected: ${SHA256_EXPECTED}
  got:      ${SHA256_GOT}" >&2
    exit 1
  fi
  echo "[verify] OK"
else
  echo "[verify] 首次取件（pin 流程）：实测 SHA256 = ${SHA256_GOT}"
  echo "        请将此值回填本脚本 SHA256_EXPECTED 后提交（版本 pin 闭环）"
fi

tar -xzf "${OUT}" -C bin
echo "[done] $(ls bin/)"
