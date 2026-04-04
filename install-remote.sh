#!/usr/bin/env bash
set -euo pipefail

REPO="usevaulter/vaulter"
INSTALL_DIR="/usr/local/bin"

# Detect platform
OS="$(uname -s)"
ARCH="$(uname -m)"

case "${OS}-${ARCH}" in
    Linux-x86_64)   ARTIFACT="vaulter-linux-amd64" ;;
    Darwin-arm64)   ARTIFACT="vaulter-macos-arm64" ;;
    Darwin-x86_64)  ARTIFACT="vaulter-macos-amd64" ;;
    *)
        echo "error: unsupported platform ${OS}-${ARCH}"
        exit 1
        ;;
esac

# Get latest version or use provided one
VERSION="${1:-$(curl -sL "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name"' | cut -d'"' -f4)}"

if [ -z "${VERSION}" ]; then
    echo "error: could not determine latest version"
    exit 1
fi

URL="https://github.com/${REPO}/releases/download/${VERSION}/${ARTIFACT}.tar.gz"
TMP_DIR="$(mktemp -d)"

echo "Downloading vaulter ${VERSION} for ${OS}-${ARCH}..."
curl -sL "${URL}" -o "${TMP_DIR}/${ARTIFACT}.tar.gz"

echo "Verifying checksum..."
curl -sL "${URL}.sha256" -o "${TMP_DIR}/${ARTIFACT}.tar.gz.sha256"
(cd "${TMP_DIR}" && shasum -a 256 -c "${ARTIFACT}.tar.gz.sha256")

echo "Installing to ${INSTALL_DIR}..."
tar -xzf "${TMP_DIR}/${ARTIFACT}.tar.gz" -C "${TMP_DIR}"
sudo install -m 755 "${TMP_DIR}/vaulter" "${INSTALL_DIR}/vaulter"

rm -rf "${TMP_DIR}"

echo "vaulter ${VERSION} installed to ${INSTALL_DIR}/vaulter"
vaulter init 2>/dev/null || true
