#!/usr/bin/env bash
set -euo pipefail

INSTALL_DIR="/usr/local/bin"

echo "Building vaulter in release mode..."
cargo build --release


echo "Installing vaulter to ${INSTALL_DIR}..."
sudo cp target/release/vaulter "${INSTALL_DIR}/vaulter"
sudo chmod +x "${INSTALL_DIR}/vaulter"

echo "Initializing vaulter..."
vaulter init

echo "Done. vaulter $(vaulter --version 2>/dev/null || echo 'v0.1.0') installed to ${INSTALL_DIR}/vaulter"
