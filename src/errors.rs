use std::fmt;

#[derive(Debug)]
pub enum VaulterError {
    Db(sqlx::Error),
    Migrate(sqlx::migrate::MigrateError),
    Io(std::io::Error),
    NotInitialized,
    VaultNotFound(String),
    VaultAlreadyExists(String),
    CannotDeleteDefault,
    NoHomeDir,
    NoCwd,
    InvalidSetArgs,
}

impl fmt::Display for VaulterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VaulterError::Db(e) => write!(f, "database error: {e}"),
            VaulterError::Migrate(e) => write!(f, "migration error: {e}"),
            VaulterError::Io(e) => write!(f, "io error: {e}"),
            VaulterError::NotInitialized => write!(
                f,
                "vaulter is not initialized. Run `vaulter init` first"
            ),
            VaulterError::VaultNotFound(name) => write!(f, "vault '{name}' not found"),
            VaulterError::VaultAlreadyExists(name) => {
                write!(f, "vault '{name}' already exists")
            }
            VaulterError::CannotDeleteDefault => {
                write!(f, "cannot delete the default vault")
            }
            VaulterError::NoHomeDir => write!(f, "could not determine home directory"),
            VaulterError::NoCwd => write!(f, "could not determine current directory"),
            VaulterError::InvalidSetArgs => write!(
                f,
                "usage: vaulter set KEY VALUE or vaulter set KEY=VALUE [KEY2=VALUE2 ...]"
            ),
        }
    }
}

impl From<sqlx::Error> for VaulterError {
    fn from(e: sqlx::Error) -> Self {
        VaulterError::Db(e)
    }
}

impl From<sqlx::migrate::MigrateError> for VaulterError {
    fn from(e: sqlx::migrate::MigrateError) -> Self {
        VaulterError::Migrate(e)
    }
}

impl From<std::io::Error> for VaulterError {
    fn from(e: std::io::Error) -> Self {
        VaulterError::Io(e)
    }
}

pub type Result<T> = std::result::Result<T, VaulterError>;
