//! Maps `sqlx` errors to wire-safe [`CoreError`]s.
//!
//! Driver detail (table names, SQL, connection state) is logged server-side and
//! never placed on the wire — a client sees only a stable [`ClientCode`]. This
//! is the "never leak internal detail in an error" rule, enforced in one place.
//!
//! [`ClientCode`]: omm_errors::ClientCode

use omm_errors::CoreError;

/// Translate a `sqlx::Error` into a stable, client-safe [`CoreError`].
pub(crate) fn map_sqlx(err: sqlx::Error) -> CoreError {
    match err {
        sqlx::Error::RowNotFound => CoreError::NotFound("row".into()),
        sqlx::Error::Database(db) if db.is_unique_violation() => {
            CoreError::Conflict("unique constraint".into())
        }
        other => {
            tracing::error!(error = %other, "database error");
            CoreError::Internal
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use omm_errors::ClientCode;

    #[test]
    fn row_not_found_maps_to_not_found() {
        assert_eq!(
            map_sqlx(sqlx::Error::RowNotFound).code(),
            ClientCode::NotFound
        );
    }

    #[test]
    fn unknown_error_hides_detail_as_internal() {
        // A protocol error stands in for "some driver fault" — it must collapse
        // to Internal, never surfacing its message to a client.
        let err = map_sqlx(sqlx::Error::Protocol("secret internal detail".into()));
        assert_eq!(err.code(), ClientCode::Internal);
        assert_eq!(err.to_string(), "internal error");
    }
}
