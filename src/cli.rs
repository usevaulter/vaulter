use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "vaulter", about = "Environment variable manager")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize vaulter
    Init,

    /// Create a new vault
    Create { name: String },

    /// List all vaults
    List,

    /// Delete a vault
    Delete { name: String },

    /// Switch active vault for current directory
    Use { vault: String },

    /// Switch vault and export its env variables for current shell (use with eval)
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

    /// Run a command with a vault's env variables
    With {
        /// Vault name (defaults to active vault)
        vault: Option<String>,
        #[arg(last = true)]
        cmd: Vec<String>,
    },
}
