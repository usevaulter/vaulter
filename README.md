# Vaulter

A fast, local-first environment variable manager for developers.

Vaulter organizes env variables into **vaults** — named groups you can switch between per project directory. No more juggling `.env` files or leaking secrets across projects.

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

### Project Structure

```
src/
  main.rs       -- Entrypoint
  cli.rs        -- CLI definition (clap derive)
  commands.rs   -- Command handlers
  db.rs         -- SQLite operations (sqlx)
  errors.rs     -- Error types
  lib.rs        -- Library exports
migrations/
  001_init.sql  -- Initial schema
tests/
  test_db.rs    -- Integration tests
```

### Guidelines

- Keep it simple. This is a CLI tool, not a framework.
- All DB logic goes in `db.rs`, command logic in `commands.rs`.
- Add tests for new features in `tests/test_db.rs`.
- Schema changes go in new migration files (`migrations/002_*.sql`, etc.).
- Run `cargo test -- --test-threads=1` before submitting a PR.

## License

MIT
