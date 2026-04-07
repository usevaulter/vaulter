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

# Ask about shell completions
SHELL_NAME="$(basename "${SHELL}")"

prompt_yn() {
  local question="$1" answer=""
  printf "%s [Y/n] " "${question}"
  read -r answer
  answer="${answer:-y}"
  [[ "${answer}" =~ ^[Yy]$ ]]
}

install_completions() {
  case "${SHELL_NAME}" in
    zsh)
      mkdir -p "${HOME}/.zfunc"
      "${INSTALL_DIR}/vaulter" completions zsh > "${HOME}/.zfunc/_vaulter"
      if ! grep -q "vaulter completions" "${HOME}/.zshrc" 2>/dev/null; then
        cat >> "${HOME}/.zshrc" <<'EOF'

# vaulter completions
fpath=(~/.zfunc $fpath)
autoload -Uz compinit && compinit
EOF
        echo "Zsh completions installed to ~/.zfunc/_vaulter (added fpath to ~/.zshrc)"
      else
        echo "Zsh completions installed to ~/.zfunc/_vaulter"
      fi
      ;;
    bash)
      if ! grep -q "vaulter completions" "${HOME}/.bashrc" 2>/dev/null; then
        cat >> "${HOME}/.bashrc" <<'EOF'

# vaulter completions
source <(vaulter completions bash)
EOF
        echo "Bash completions added to ~/.bashrc"
      else
        echo "Bash completions already sourced from ~/.bashrc"
      fi
      ;;
    fish)
      mkdir -p "${HOME}/.config/fish/completions"
      "${INSTALL_DIR}/vaulter" completions fish > "${HOME}/.config/fish/completions/vaulter.fish"
      echo "Fish completions installed to ~/.config/fish/completions/vaulter.fish"
      ;;
    *)
      echo "Shell '${SHELL_NAME}' not supported for completion auto-install."
      return
      ;;
  esac
}

if prompt_yn "Install shell completions for ${SHELL_NAME}?"; then
  install_completions
else
  echo "Skipped completions. Install later with: vaulter completions ${SHELL_NAME}"
fi
