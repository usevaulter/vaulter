use std::ffi::OsStr;
use std::fmt;
use std::ops::Deref;

/// Environment variable key (e.g., "API_KEY", "DATABASE_URL")
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, sqlx::Type)]
#[sqlx(transparent)]
pub struct Key(String);

impl Key {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Deref for Key {
    type Target = str;
    fn deref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for Key {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for Key {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl AsRef<str> for Key {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl AsRef<OsStr> for Key {
    fn as_ref(&self) -> &OsStr {
        self.0.as_ref()
    }
}

/// Environment variable value
#[derive(Debug, Clone, PartialEq, Eq, sqlx::Type)]
#[sqlx(transparent)]
pub struct Value(String);

impl Value {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Deref for Value {
    type Target = str;
    fn deref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl AsRef<str> for Value {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl AsRef<OsStr> for Value {
    fn as_ref(&self) -> &OsStr {
        self.0.as_ref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, sqlx::FromRow)]
pub struct EnvVar {
    pub key: Key,
    pub value: Value,
}

impl EnvVar {
    pub fn new(key: impl Into<Key>, value: impl Into<Value>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
        }
    }
}
