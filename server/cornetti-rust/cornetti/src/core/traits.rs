use crate::core::models::{CornettiError, CornettiResult};
use std::{future::Future, time::Duration};

/// Converts `self` into `T`.
///
/// Blanket-implemented for any type that implements `From<T>`.
pub trait To<T> {
    fn to(self) -> T;
}

/// Blanket implementation: `To<U>` for any `T` where `U: From<T>`.
impl<T, U> To<U> for T
where
    U: From<T>,
{
    fn to(self) -> U {
        U::from(self)
    }
}

/// Marker trait for models with a default constructor.
pub trait BaseModel {
    fn new() -> Self;
}

/// Module metadata trait.
///
/// Used to register modules with their name, version, and permission set.
pub trait BaseModule {
    /// The unique module name.
    fn module_name() -> &'static str;
    /// The current module version (used for incremental migrations).
    fn module_version() -> i32;
    /// The static list of permissions this module declares.
    fn module_permissions() -> &'static [&'static str];
    /// Returns permissions as owned `String` values.
    fn module_permissions_strings() -> Vec<String> {
        Self::module_permissions()
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<String>>()
    }
}

/// Exponential backoff retry strategy for transient repository errors.
///
/// Default configuration: 3 attempts, 100 ms base delay, 1.5× exponential factor.
/// Only errors with HTTP status 503 are considered transient by default.
///
/// # Cancellation
///
/// Not cancel-safe: dropping the future returned by [`retry_transient`] mid-retry
/// may leave the wrapped operation partially executed. Callers using
/// `tokio::select!` should take this into account.
///
/// [`retry_transient`]: RepositoryRetry::retry_transient
pub trait RepositoryRetry {
    /// Number of total attempts (default: 3).
    fn retry_attempts(&self) -> usize {
        3
    }
    /// Base delay between retries (default: 100 ms).
    fn retry_delay(&self) -> Duration {
        Duration::from_millis(100)
    }
    /// Whether the error is transient and worth retrying (default: status 503).
    fn is_transient_error(&self, err: &CornettiError) -> bool {
        err.status == 503
    }
    /// Executes `op` with exponential backoff (1.5× factor) up to `retry_attempts`.
    ///
    /// Only retries when `is_transient_error` returns `true`.
    /// Returns the last transient error after exhausting all attempts.
    fn retry_transient<T, Fut, Op>(&self, mut op: Op) -> impl Future<Output = CornettiResult<T>>
    where
        Op: FnMut() -> Fut,
        Fut: Future<Output = CornettiResult<T>>,
    {
        let attempts = self.retry_attempts().max(1);
        let base_delay = self.retry_delay();

        async move {
            let mut last_err = None;

            for attempt in 0..attempts {
                match op().await {
                    Ok(result) => return Ok(result),
                    Err(err) if self.is_transient_error(&err) => {
                        last_err = Some(err);
                        if attempt + 1 < attempts {
                            let delay = base_delay.mul_f32(1.5f32.powi(attempt as i32));
                            log::warn!(
                                "Transient repository error, retrying {}/{}: {} (wait {:?})",
                                attempt + 1,
                                attempts,
                                last_err.as_ref().unwrap().detail,
                                delay
                            );
                            tokio::time::sleep(delay).await;
                        }
                    }
                    Err(err) => return Err(err),
                }
            }

            log::error!(
                "Exhausted {} retry attempts: {}",
                attempts,
                last_err.as_ref().unwrap().detail
            );
            Err(last_err.unwrap())
        }
    }
}
