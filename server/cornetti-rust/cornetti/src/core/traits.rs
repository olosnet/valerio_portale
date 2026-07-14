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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::CornettiError;
    use std::{
        cell::Cell,
        rc::Rc,
        time::Duration,
    };

    type TransientCheck = Rc<dyn Fn(&CornettiError) -> bool>;

    struct TestRepo {
        custom_retry_attempts: Option<usize>,
        custom_retry_delay: Option<Duration>,
        custom_transient_check: Option<TransientCheck>,
    }

    impl RepositoryRetry for TestRepo {
        fn retry_attempts(&self) -> usize {
            self.custom_retry_attempts.unwrap_or(3)
        }
        fn retry_delay(&self) -> Duration {
            self.custom_retry_delay.unwrap_or(Duration::from_millis(100))
        }
        fn is_transient_error(&self, err: &CornettiError) -> bool {
            if let Some(ref check) = self.custom_transient_check {
                check(err)
            } else {
                err.status == 503
            }
        }
    }

    fn transient_err() -> CornettiError {
        CornettiError {
            status: 503,
            detail: "transient".into(),
        }
    }

    fn non_transient_err() -> CornettiError {
        CornettiError {
            status: 500,
            detail: "non transient".into(),
        }
    }

    #[test]
    fn to_blanket_impl_converts() {
        #[derive(Debug, PartialEq)]
        struct FromType(i32);
        struct ToType(i32);
        impl From<FromType> for ToType {
            fn from(f: FromType) -> Self {
                ToType(f.0)
            }
        }
        let from_val = FromType(42);
        let to_val: ToType = from_val.to();
        assert_eq!(to_val.0, 42);
    }

    #[test]
    fn base_module_permissions_strings_empty() {
        struct EmptyMod;
        impl BaseModule for EmptyMod {
            fn module_name() -> &'static str {
                "empty"
            }
            fn module_version() -> i32 {
                1
            }
            fn module_permissions() -> &'static [&'static str] {
                &[]
            }
        }
        assert!(EmptyMod::module_permissions_strings().is_empty());
    }

    #[test]
    fn base_module_permissions_strings_non_empty() {
        struct ModWithPerms;
        impl BaseModule for ModWithPerms {
            fn module_name() -> &'static str {
                "mod"
            }
            fn module_version() -> i32 {
                1
            }
            fn module_permissions() -> &'static [&'static str] {
                &["read", "write"]
            }
        }
        let perms = ModWithPerms::module_permissions_strings();
        assert_eq!(perms.len(), 2);
        assert_eq!(perms[0], "read");
        assert_eq!(perms[1], "write");
    }

    #[test]
    fn repository_retry_default_attempts() {
        struct DefaultRepo;
        impl RepositoryRetry for DefaultRepo {}
        assert_eq!(DefaultRepo.retry_attempts(), 3);
    }

    #[test]
    fn repository_retry_default_delay() {
        struct DefaultRepo;
        impl RepositoryRetry for DefaultRepo {}
        assert_eq!(DefaultRepo.retry_delay(), Duration::from_millis(100));
    }

    #[test]
    fn repository_retry_default_is_transient() {
        struct DefaultRepo;
        impl RepositoryRetry for DefaultRepo {}
        assert!(DefaultRepo.is_transient_error(&transient_err()));
        assert!(!DefaultRepo.is_transient_error(&non_transient_err()));
    }

    #[test]
    fn repository_retry_custom_attempts() {
        let repo = TestRepo {
            custom_retry_attempts: Some(5),
            custom_retry_delay: None,
            custom_transient_check: None,
        };
        assert_eq!(repo.retry_attempts(), 5);
    }

    #[test]
    fn repository_retry_custom_delay() {
        let repo = TestRepo {
            custom_retry_attempts: None,
            custom_retry_delay: Some(Duration::from_millis(50)),
            custom_transient_check: None,
        };
        assert_eq!(repo.retry_delay(), Duration::from_millis(50));
    }

    #[test]
    fn repository_retry_custom_transient_check() {
        let check: TransientCheck = Rc::new(|e| e.status == 408);
        let repo = TestRepo {
            custom_retry_attempts: None,
            custom_retry_delay: None,
            custom_transient_check: Some(check),
        };
        assert!(repo.is_transient_error(&CornettiError {
            status: 408,
            detail: "timeout".into(),
        }));
        assert!(!repo.is_transient_error(&transient_err()));
    }

    #[tokio::test]
    async fn retry_transient_success_first_try() {
        let calls = Rc::new(Cell::new(0));
        let calls_clone = calls.clone();
        let repo = TestRepo {
            custom_retry_attempts: None,
            custom_retry_delay: None,
            custom_transient_check: None,
        };
        let result = repo
            .retry_transient(move || {
                let c = calls_clone.clone();
                async move {
                    c.set(c.get() + 1);
                    Ok::<i32, CornettiError>(42)
                }
            })
            .await;
        assert_eq!(result.unwrap(), 42);
        assert_eq!(calls.get(), 1);
    }

    #[tokio::test]
    async fn retry_transient_fail_non_transient() {
        let calls = Rc::new(Cell::new(0));
        let calls_clone = calls.clone();
        let repo = TestRepo {
            custom_retry_attempts: None,
            custom_retry_delay: None,
            custom_transient_check: None,
        };
        let result = repo
            .retry_transient(move || {
                let c = calls_clone.clone();
                async move {
                    c.set(c.get() + 1);
                    Err::<i32, CornettiError>(non_transient_err())
                }
            })
            .await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().status, 500);
        assert_eq!(calls.get(), 1);
    }

    #[tokio::test]
    async fn retry_transient_success_after_retry() {
        let calls = Rc::new(Cell::new(0));
        let calls_clone = calls.clone();
        let repo = TestRepo {
            custom_retry_attempts: None,
            custom_retry_delay: Some(Duration::from_millis(1)),
            custom_transient_check: None,
        };
        let result = repo
            .retry_transient(move || {
                let c = calls_clone.clone();
                async move {
                    let n = c.get();
                    c.set(n + 1);
                    if n < 2 {
                        Err::<i32, CornettiError>(transient_err())
                    } else {
                        Ok(99)
                    }
                }
            })
            .await;
        assert_eq!(result.unwrap(), 99);
        assert_eq!(calls.get(), 3);
    }

    #[tokio::test]
    async fn retry_transient_exhaust_all_retries() {
        let calls = Rc::new(Cell::new(0));
        let calls_clone = calls.clone();
        let repo = TestRepo {
            custom_retry_attempts: Some(3),
            custom_retry_delay: Some(Duration::from_millis(1)),
            custom_transient_check: None,
        };
        let result = repo
            .retry_transient(move || {
                let c = calls_clone.clone();
                async move {
                    c.set(c.get() + 1);
                    Err::<i32, CornettiError>(transient_err())
                }
            })
            .await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().status, 503);
        assert_eq!(calls.get(), 3);
    }

    #[tokio::test]
    async fn retry_transient_mixed_transient_then_non_transient() {
        let calls = Rc::new(Cell::new(0));
        let calls_clone = calls.clone();
        let repo = TestRepo {
            custom_retry_attempts: Some(5),
            custom_retry_delay: Some(Duration::from_millis(1)),
            custom_transient_check: None,
        };
        let result = repo
            .retry_transient(move || {
                let c = calls_clone.clone();
                async move {
                    let n = c.get();
                    c.set(n + 1);
                    if n == 0 {
                        Err::<i32, CornettiError>(transient_err())
                    } else {
                        Err(non_transient_err())
                    }
                }
            })
            .await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().status, 500);
        assert_eq!(calls.get(), 2);
    }

    #[tokio::test]
    async fn retry_transient_single_attempt() {
        let calls = Rc::new(Cell::new(0));
        let calls_clone = calls.clone();
        let repo = TestRepo {
            custom_retry_attempts: Some(1),
            custom_retry_delay: Some(Duration::from_millis(1)),
            custom_transient_check: None,
        };
        let result = repo
            .retry_transient(move || {
                let c = calls_clone.clone();
                async move {
                    c.set(c.get() + 1);
                    Err::<i32, CornettiError>(transient_err())
                }
            })
            .await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().status, 503);
        assert_eq!(calls.get(), 1);
    }
}
