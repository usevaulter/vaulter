# Vaulter

A fast, local-first environment variable manager for developers.

Vaulter organizes env variables into **vaults** — named groups you can switch between per project directory. No more juggling `.env` files or leaking secrets across projects.


## Roadmap

### Core
- [x] `vaulter init` — auto-initialize on first use
- [x] `vaulter create / list / delete` — vault management
- [x] `vaulter use` — switch active vault per directory
- [x] `vaulter set / get / unset` — variable CRUD
- [x] `vaulter set KEY=val KEY2=val2` — multi-set support
- [x] `vaulter show` — display vault variables
- [x] `vaulter import` — import from `.env` files
- [x] `vaulter export` — export as shell statements
- [x] `vaulter switch` — switch vault and export for shell eval
- [x] `vaulter with` — run commands with vault env injected

### Phase 1 — Versioning
- [ ] `vaulter log` — history of changes per vault
- [ ] `vaulter diff` — diff between vaults or versions
- [ ] `vaulter rollback` — restore a previous state
- [ ] Snapshot tagging
- [ ] Autocompletion

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

## Installation

### From source

```bash
git clone https://github.com/usevaulter/vaulter.git
cd vaulter
./install.sh
```

### One-liner

```bash
curl -sL https://raw.githubusercontent.com/usevaulter/vaulter/main/install-remote.sh | bash
```

## Quick Start

```bash
# Create a vault
vaulter create myproject

# Switch to it (remembered per directory)
vaulter use myproject

# Set variables
vaulter set API_KEY sk-123
vaulter set DB_HOST=localhost DB_PORT=5432

# Import from an existing .env file
vaulter import .env

# Run your app with the vault's variables
vaulter with myproject -- npm start
```

## Commands

### Vaults

```bash
# Create a vault
vaulter create staging

# List all vaults (* marks active for current directory)
vaulter list

# Switch active vault for current directory
vaulter use staging

# Switch and export variables for your shell
eval "$(vaulter switch staging)"

# Delete a vault
vaulter delete staging
```

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
# Run a command with a vault's variables injected
vaulter with staging -- npm start
vaulter with production -- docker compose up

# Use the active vault
vaulter with -- python app.py
```

The child process inherits your current environment plus the vault's variables. Vaulter exits with the child's exit code.

## Per-Directory Active Vault

Vaulter remembers which vault is active **per directory**. This means:

```bash
cd ~/projects/frontend
vaulter use frontend-dev    # remembered for this directory

cd ~/projects/api
vaulter use api-staging     # different vault for this directory

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
      command vaulter "$@" && eval "$(command vaulter export)"
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

## Environment Variables

| Variable | Description |
|---|---|
| `VAULTER_HOME` | Override the default `~/.vaulter/` directory |

## Data Storage

All data is stored locally in a SQLite database at `~/.vaulter/vaulter.db` (or `$VAULTER_HOME/vaulter.db`). Values are stored in plaintext. The database is auto-initialized on first use. Schema migrations are handled automatically via sqlx.

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
