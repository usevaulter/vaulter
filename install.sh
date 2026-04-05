#!/usr/bin/env bash
set -euo pipefail

INSTALL_DIR="${HOME}/.local/bin"

echo "Building vaulter in release mode..."
cargo build --release

echo "Installing vaulter to ${INSTALL_DIR}..."
mkdir -p "${INSTALL_DIR}"
cp target/release/vaulter "${INSTALL_DIR}/vaulter"
chmod +x "${INSTALL_DIR}/vaulter"

case ":${PATH}:" in
  *":${INSTALL_DIR}:"*) ;;
  *)
    echo "Warning: ${INSTALL_DIR} is not in your PATH."
    echo "Add this to your shell profile (e.g. ~/.zshrc, ~/.bashrc):"
    echo "  export PATH=\"\$HOME/.local/bin:\$PATH\""
    ;;
esac

echo "Initializing vaulter..."
"${INSTALL_DIR}/vaulter" init

echo "Done. vaulter $("${INSTALL_DIR}/vaulter" --version 2>/dev/null || echo 'v0.1.0') installed to ${INSTALL_DIR}/vaulter"
