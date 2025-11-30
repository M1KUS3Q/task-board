/// Trait for tracing SQLx errors by converting them into string errors while logging.
///
/// Provides methods to map a `Result<T, sqlx::Error>` into `Result<T, String>`,
/// logging the error with different severity levels.
///
/// # Type Parameters
///
/// * `Ok` - The success type of the result after tracing the error.
///
/// # Examples
///
/// ```ignore
/// let result: Result<i32, sqlx::Error> = Err(sqlx::Error::RowNotFound);
/// let traced = result.trace_err("failed to fetch row");
/// // Logs an error: "failed to fetch row: RowNotFound"
/// assert_eq!(traced,Err("failed to fetch row: RowNotFound".to_string()));
/// ```
pub trait TraceError {
    /// The success type to return on `Ok`.
    type Ok;

    /// Logs the error at the `error` level with the given message prefix,
    /// then converts the error into a `String` containing the prefix and error details.
    ///
    /// # Arguments
    ///
    /// * `self` - The `Result<T, sqlx::Error>` to be mapped.
    /// * `msg` - A message prefix implementing `ToString` for context in the log and returned error.
    ///
    /// # Returns
    ///
    /// * `Ok(value)` if the original result is `Ok(value)`.
    /// * `Err(String)` containing `msg: original_error` if the original result is `Err`.
    fn trace_err<S: ToString>(self, msg: S) -> Result<Self::Ok, String>;

    /// Logs the error at the `warn` level with the given message prefix,
    /// then converts the error into a `String` containing the prefix and error details.
    ///
    /// # Arguments
    ///
    /// * `self` - The `Result<T, sqlx::Error>` to be mapped.
    /// * `msg` - A message prefix implementing `ToString` for context in the log and returned error.
    ///
    /// # Returns
    ///
    /// * `Ok(value)` if the original result is `Ok(value)`.
    /// * `Err(String)` containing `msg: original_error` if the original result is `Err`.
    fn trace_warn<S: ToString>(self, msg: S) -> Result<Self::Ok, String>;
}

/// Implements `TraceError` for `Result<T, sqlx::Error>`,
/// mapping SQLx errors into string errors while logging at the appropriate level.
///
/// # Type Parameters
///
/// * `T` - The success type of the original `Result`.
///
/// # Behavior
///
/// - `trace_err` logs with `tracing::error!` and returns `Err(format!("{msg}: {e}"))`.
/// - `trace_warn` logs with `tracing::warn!` and returns `Err(format!("{msg}: {e}"))`.
impl<T> TraceError for Result<T, sqlx::Error> {
    type Ok = T;

    /// See `TraceError::trace_err`.
    fn trace_err<S: ToString>(self, msg: S) -> Result<T, String> {
        self.map_err(|e| {
            let msg = msg.to_string();
            tracing::error!("{msg}: {e}");
            format!("{msg}: {e}")
        })
    }

    /// See `TraceError::trace_warn`.
    fn trace_warn<S: ToString>(self, msg: S) -> Result<T, String> {
        self.map_err(|e| {
            let msg = msg.to_string();
            tracing::warn!("{msg}: {e}");
            format!("{msg}: {e}")
        })
    }
}
