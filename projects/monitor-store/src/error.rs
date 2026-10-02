use std::fmt;

/// A persistence operation failed.
///
/// This used to be a struct holding one `String`, which forced every caller to
/// parse prose to find out what happened, and made "target not found"
/// indistinguishable from "the disk is on fire". Matching on the variant is now
/// how those two are told apart, which is exactly what the Web layer needs in
/// order to answer `404` instead of `500`.
#[derive(Debug)]
pub enum StoreError {
    /// No target with the requested identifier exists.
    NotFound {
        /// The identifier the caller asked for.
        target_id: i64,
    },
    /// A result was saved against a target that did not produce it.
    TargetMismatch {
        /// The identifier the caller tried to save the result under.
        target_id: i64,
    },
    /// A stored row could not be turned back into a domain value.
    ///
    /// The row was written by an older, or a buggier, version of the program.
    Corrupt {
        /// What was wrong with the stored data.
        detail: String,
    },
    /// The schema could not be created or upgraded.
    Migration {
        /// What the migration runner reported.
        detail: String,
    },
    /// The database itself rejected or failed the operation.
    Database(sqlx::Error),
}

impl fmt::Display for StoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound { target_id } => write!(formatter, "target {target_id} was not found"),
            Self::TargetMismatch { target_id } => {
                write!(
                    formatter,
                    "check result does not belong to target {target_id}"
                )
            }
            Self::Corrupt { detail } => write!(formatter, "stored data is invalid: {detail}"),
            Self::Migration { detail } => write!(formatter, "database migration failed: {detail}"),
            Self::Database(error) => write!(formatter, "database error: {error}"),
        }
    }
}

impl std::error::Error for StoreError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Database(error) => Some(error),
            Self::NotFound { .. }
            | Self::TargetMismatch { .. }
            | Self::Corrupt { .. }
            | Self::Migration { .. } => None,
        }
    }
}

impl From<sqlx::Error> for StoreError {
    fn from(error: sqlx::Error) -> Self {
        Self::Database(error)
    }
}

impl StoreError {
    /// Wraps any error raised while turning a row into a domain value.
    pub(crate) fn corrupt(error: impl fmt::Display) -> Self {
        Self::Corrupt {
            detail: error.to_string(),
        }
    }
}
