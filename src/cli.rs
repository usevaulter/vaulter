use clap::{Parser, Subcommand};
use clap_complete::Shell;

#[derive(Parser)]
#[command(name = "vaulter", version, about = "Environment variable manager")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize vaulter
    Init,

    /// Show debug information about vaulter installation
    #[command(visible_alias = "info")]
    Debug,

    /// Create a new vault
    Create { name: String },

    /// List all vaults
    #[command(visible_alias = "ls")]
    List,

    /// Delete a vault
    #[command(visible_alias = "rm")]
    Delete { name: String },

    /// Switch active vault for current directory
    #[command(visible_alias = "select")]
    Use { vault: String },

    /// Switch vault and export its env variables for current shell (use with eval)
    #[command(visible_alias = "sw")]
    Switch {
        /// Vault name (defaults to active vault)
        name: Option<String>,
    },

    /// Set environment variables (KEY VALUE or KEY=VALUE KEY2=VALUE2 ...)
    Set {
        /// KEY VALUE or KEY=VALUE pairs
        args: Vec<String>,
        #[arg(long)]
        vault: Option<String>,
    },

    /// Get an environment variable value
    Get {
        key: String,
        #[arg(long)]
        vault: Option<String>,
    },

    /// Show all variables in a vault
    #[command(visible_alias = "print")]
    Show {
        /// Vault name (defaults to active vault)
        vault: Option<String>,
    },

    /// Remove an environment variable
    Unset {
        key: String,
        #[arg(long)]
        vault: Option<String>,
    },

    /// Export variables as shell export statements
    Export {
        #[arg(long)]
        vault: Vec<String>,
    },

    /// Import variables from a .env file
    Import {
        /// Path to the .env file
        file: String,
        #[arg(long)]
        vault: Option<String>,
    },

    /// Print a shell completion script to stdout
    ///
    /// Example: `vaulter completions zsh > ~/.zfunc/_vaulter`
    Completions { shell: Shell },

    /// Internal: emit values for shell completion (one per line)
    #[command(name = "_complete", hide = true)]
    InternalComplete {
        /// What to complete: "vaults", "vars", or "shells"
        kind: String,
        /// Vault name (for kind="vars"); defaults to active vault
        #[arg(long)]
        vault: Option<String>,
    },

    /// Run a command with a vault's env variables
    ///
    /// Uses active vault by default. To target a specific vault without
    /// switching, use: `vaulter run with <vault> -- <command>`
    Run {
        /// Optional "with <vault>" prefix to target a specific vault
        args: Vec<String>,
        #[arg(last = true)]
        cmd: Vec<String>,
    },
}
