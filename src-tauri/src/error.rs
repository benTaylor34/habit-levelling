use serde::{Serialize, Serializer};
use std::fmt;

/// Errors returned from commands. Serialised to a plain message for the frontend.
#[derive(Debug)]
pub enum AppError {
    Db(sqlx::Error),
    NotFound(i64),
    Invalid(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Db(e) => write!(f, "Database error: {e}"),
            AppError::NotFound(id) => write!(f, "Habit {id} not found"),
            AppError::Invalid(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for AppError {}

impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        AppError::Db(e)
    }
}

impl Serialize for AppError {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;
