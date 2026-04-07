#!/usr/bin/env bash
set -euo pipefail

REPO="usevaulter/vaulter"
INSTALL_DIR="${HOME}/.local/bin"

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
mkdir -p "${INSTALL_DIR}"
tar -xzf "${TMP_DIR}/${ARTIFACT}.tar.gz" -C "${TMP_DIR}"
install -m 755 "${TMP_DIR}/vaulter" "${INSTALL_DIR}/vaulter"

rm -rf "${TMP_DIR}"

echo "vaulter ${VERSION} installed to ${INSTALL_DIR}/vaulter"
vaulter init 2>/dev/null || true

# Detect shell and install hook
SHELL_NAME="$(basename "${SHELL}")"

ZSH_HOOK='
# vaulter shell integration
autoload -U add-zsh-hook

vaulter() {
  case "$1" in
    use|switch)
      command vaulter "$@" && eval "$(command vaulter export)"
      ;;
    *)
      command vaulter "$@"
      ;;
  esac
}

_vaulter_chpwd() { eval "$(command vaulter export 2>/dev/null)"; }
add-zsh-hook chpwd _vaulter_chpwd
_vaulter_chpwd'

BASH_HOOK='
# vaulter shell integration
vaulter() {
  case "$1" in
    use|switch)
      command vaulter "$@" && eval "$(command vaulter export)"
      ;;
    *)
      command vaulter "$@"
      ;;
  esac
}

_vaulter_chpwd() { eval "$(command vaulter export 2>/dev/null)"; }
if [[ -n "${PROMPT_COMMAND}" ]]; then
  PROMPT_COMMAND="_vaulter_chpwd;${PROMPT_COMMAND}"
else
  PROMPT_COMMAND="_vaulter_chpwd"
fi
_vaulter_chpwd'

FISH_HOOK='
# vaulter shell integration
function vaulter
  switch $argv[1]
    case use switch
      command vaulter $argv; and eval (command vaulter export 2>/dev/null)
    case "*"
      command vaulter $argv
  end
end

function _vaulter_chpwd --on-variable PWD
  eval (command vaulter export 2>/dev/null)
end
_vaulter_chpwd'

case "${SHELL_NAME}" in
  zsh)
    RC_FILE="${HOME}/.zshrc"
    HOOK="${ZSH_HOOK}"
    ;;
  bash)
    RC_FILE="${HOME}/.bashrc"
    HOOK="${BASH_HOOK}"
    ;;
  fish)
    RC_FILE="${HOME}/.config/fish/conf.d/vaulter.fish"
    mkdir -p "$(dirname "${RC_FILE}")"
    HOOK="${FISH_HOOK}"
    ;;
  *)
    echo "Shell '${SHELL_NAME}' not supported for auto-hook. Add vaulter export to your shell config manually."
    exit 0
    ;;
esac

if ! grep -q "vaulter shell integration" "${RC_FILE}" 2>/dev/null; then
  echo "${HOOK}" >> "${RC_FILE}"
  echo "Shell hook added to ${RC_FILE}. Restart your shell or run: source ${RC_FILE}"
else
  echo "Shell hook already present in ${RC_FILE}"
fi

# Ask about shell completions
prompt_yn() {
  local question="$1" answer=""
  if [ -r /dev/tty ]; then
    printf "%s [Y/n] " "${question}" > /dev/tty
    read -r answer < /dev/tty
  fi
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
  esac
}

if prompt_yn "Install shell completions for ${SHELL_NAME}?"; then
  install_completions
else
  echo "Skipped completions. Install later with: vaulter completions ${SHELL_NAME}"
fi
