# Vaulter

A fast, local-first environment variable manager for developers.

Vaulter organizes env variables into **vaults** — named groups you can switch between per project directory. No more juggling `.env` files or leaking secrets across projects.

## Installation

### One-liner

```bash
curl -sL https://raw.githubusercontent.com/usevaulter/vaulter/main/install-remote.sh | bash
```

### From source

```bash
git clone https://github.com/usevaulter/vaulter.git
cd vaulter
./install.sh
```


## Quick Start

```bash
# Create a vault
vaulter create myproject

# Switch to it (remembered per directory + loads env)
eval "$(vaulter switch myproject)"

# Set variables
vaulter set API_KEY sk-123
vaulter set DB_HOST=localhost DB_PORT=5432

# Import from an existing .env file
vaulter import .env

# Run your app with the active vault's variables
vaulter run -- npm start

# Or target a specific vault without switching
vaulter run with myproject -- npm start
```

## Commands

### Vaults

```bash
# Create a vault
vaulter create staging

# List all vaults (* marks active for current directory)
vaulter list

# Load a vault's variables into the current shell (no DB change)
eval "$(vaulter use staging)"

# Switch to a vault — set as active in DB and load its variables
eval "$(vaulter switch staging)"

# Delete a vault
vaulter delete staging
```

#### `use` vs `switch`

Both commands export a vault's variables, but differ in persistence:

| | `vaulter use <vault>` | `vaulter switch <vault>` |
|---|---|---|
| Prints `export KEY=value` statements | yes | yes |
| Updates active vault (per-directory) in DB | no | yes |
| Needs `eval "$(...)"` wrapper | yes | yes |

Use `use` when you want a one-off env load without changing which vault is active. Use `switch` when you want to change the active vault for this directory and load its variables.

### Variables

```bash
# Set one variable
vaulter set KEY value

# Set multiple variables at once
vaulter set KEY1=value1 KEY2=value2

# Get a variable
vaulter get KEY

# Show all variables in the active vault
vaulter show

# Show all variables in a specific vault
vaulter show staging

# Remove a variable
vaulter unset KEY

# Target a specific vault
vaulter set KEY value --vault staging
vaulter get KEY --vault staging
```

### Import

```bash
# Import from a .env file into the active vault
vaulter import .env

# Import into a specific vault
vaulter import .env --vault production
```

Supported `.env` formats:
- `KEY=value`
- `KEY="quoted value"`
- `KEY='single quoted'`
- `export KEY=value`
- Comments (`#`) and blank lines are skipped

### Export

```bash
# Export active vault as shell statements
vaulter export

# Export specific vaults (later vaults override earlier ones)
vaulter export --vault default --vault staging

# Load into current shell
eval "$(vaulter export)"
```

### Run Commands

```bash
# Run a command with the active vault's variables injected
vaulter run -- python app.py

# Target a specific vault without switching the active one
vaulter run with staging -- npm start
vaulter run with production -- docker compose up
```

The child process inherits your current environment plus the vault's variables. Vaulter exits with the child's exit code.

## Per-Directory Active Vault

Vaulter remembers which vault is active **per directory**. This means:

```bash
cd ~/projects/frontend
vaulter switch frontend-dev  # remembered for this directory

cd ~/projects/api
vaulter switch api-staging   # different vault for this directory

cd ~/projects/frontend
vaulter show                # automatically uses frontend-dev
```

## Shell Integration

Add to your `~/.zshrc` to auto-load env vars when changing directories:

```bash
autoload -U add-zsh-hook

vaulter() {
  case "$1" in
    use|switch)
      eval "$(command vaulter "$@")"
      ;;
    *)
      command vaulter "$@"
      ;;
  esac
}

_vaulter_chpwd() { eval "$(command vaulter export 2>/dev/null)"; }
add-zsh-hook chpwd _vaulter_chpwd
_vaulter_chpwd
```

## Shell Completion

Vaulter ships with shell completion including **dynamic value completion** (vault names, env keys).

### Zsh (recommended)

```bash
# Install
mkdir -p ~/.zfunc
vaulter completions zsh > ~/.zfunc/_vaulter

# Add to ~/.zshrc (if not already present)
fpath=(~/.zfunc $fpath)
autoload -Uz compinit && compinit
```

Then `vaulter use <TAB>` completes with your vault names, `vaulter get <TAB>` completes with keys in the active vault, etc.

### Bash

```bash
vaulter completions bash > /usr/local/etc/bash_completion.d/vaulter
# or source it directly from ~/.bashrc:
#   source <(vaulter completions bash)
```

### Fish

```bash
vaulter completions fish > ~/.config/fish/completions/vaulter.fish
```

Note: dynamic completion (live vault/key names) is currently zsh-only. Bash and fish get static command/flag completion.

## Command Aliases

Several commands have shorter aliases for convenience:

| Command | Alias | Example |
|---|---|---|
| `list` | `ls` | `vaulter ls` |
| `delete` | `rm` | `vaulter rm staging` |
| `show` | `print` | `vaulter print` |
| `use` | `select` | `eval "$(vaulter select staging)"` |
| `switch` | `sw` | `eval "$(vaulter sw staging)"` |
| `debug` | `info` | `vaulter info` |

## Environment Variables

| Variable | Description |
|---|---|
| `VAULTER_HOME` | Override the default `~/.vaulter/` directory |

## Data Storage

All data is stored locally in a SQLite database at `~/.vaulter/vaulter.db` (or `$VAULTER_HOME/vaulter.db`). Values are stored in plaintext. The database is auto-initialized on first use. Schema migrations are handled automatically via sqlx.

## Roadmap

### Core
- [x] `vaulter init` — auto-initialize on first use
- [x] `vaulter create / list / delete` — vault management
- [x] `vaulter use` — export a vault's variables (no DB change)
- [x] `vaulter set / get / unset` — variable CRUD
- [x] `vaulter set KEY=val KEY2=val2` — multi-set support
- [x] `vaulter show` — display vault variables
- [x] `vaulter import` — import from `.env` files
- [x] `vaulter export` — export as shell statements
- [x] `vaulter switch` — set active vault in DB and export variables
- [x] `vaulter run` — run commands with vault env injected (`run with <vault>` to target a specific vault)

### Phase 1 — Versioning
- [ ] `vaulter log` — history of changes per vault
- [ ] `vaulter diff` — diff between vaults or versions
- [ ] `vaulter rollback` — restore a previous state
- [ ] Snapshot tagging
- [x] Autocompletion (zsh with dynamic values; bash/fish static)

### Phase 2 — Clone, Merge & Multi-format
- [ ] `vaulter clone <source> <dest>` — duplicate a vault
- [ ] `vaulter merge <source> --into <dest>` — merge with conflict detection
- [ ] `vaulter rename <old> <new>`
- [ ] `vaulter export --format json/toml/yaml`

### Phase 3 — Encryption
- [ ] `vaulter set KEY --secret` — prompt without echo
- [ ] AES-256-GCM encryption at rest
- [ ] Master password with argon2 key derivation

### Phase 4 — Remote Sync
- [ ] `vaulter login` — authenticate
- [ ] Turso-backed remote sync across machines
- [ ] Team-shared vaults
- [ ] End-to-end encrypted sync

### Phase 5 — TUI
- [ ] Interactive terminal UI (ratatui)
- [ ] Browse, edit, and switch vaults visually

## Contributing
You are welcome !

### Prerequisites

- Rust 1.85+ (edition 2024)

### Setup

```bash
git clone https://github.com/usevaulter/vaulter.git
cd vaulter
cargo build
```

### Running Tests

```bash
cargo test -- --test-threads=1
```

Tests must run single-threaded because they use `VAULTER_HOME` for isolation.


## License

MIT
