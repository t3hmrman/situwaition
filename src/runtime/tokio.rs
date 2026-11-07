#![cfg(feature = "tokio")]

use std::{error::Error, future::Future};

use async_trait::async_trait;
use tokio::time::{sleep, timeout, Instant};

use crate::{SituwaitionError, Timeout, TokioAsyncSituwaition};

use super::AsyncWaiter;

#[async_trait]
#[cfg_attr(docsrs, doc(cfg(feature = "tokio")))]
impl<F, A, R, E> TokioAsyncSituwaition for AsyncWaiter<F, A, R, E>
where
    F: Future<Output = Result<R, E>> + Send,
    A: Fn() -> F + Send,
    R: Send + Sync,
    E: Error + Send + Sync,
{
    async fn exec(&mut self) -> Result<R, SituwaitionError<E>> {
        let start = Instant::now();
        let check_timeout = self.opts.timeout;
        let cooldown = self.opts.check_cooldown;

        loop {
            let fut = (self.factory)();
            let outcome = match check_timeout {
                Timeout::After(t) => timeout(t, fut).await,
                Timeout::Forever => Ok(fut.await),
            };
            match outcome {
                // Check completed in time and successfully and we can return
                Ok(Ok(v)) => return Ok(v),
                // Check timed out
                Err(_) => return Err(SituwaitionError::CheckTimeoutError),
                // Check completed in time but failed
                Ok(Err(e)) => {
                    if let Some(t) = cooldown {
                        sleep(t).await;
                    }

                    if matches!(check_timeout, Timeout::After(t) if Instant::now() - start > t) {
                        return Err(SituwaitionError::TimeoutError(e));
                    }

                    // If we got a condition failure, sleep and try again in the next loop
                    sleep(self.opts.check_interval).await;
                }
            }
        }
    }
}

/// Wait for a given function to resolve with a given result.
///
/// Returning a result (as opposed to the error) will end waiting, otherwise
/// the function will be retried up until the default timeout (see SituwaitionOpts)
#[allow(dead_code)]
#[cfg_attr(docsrs, doc(cfg(feature = "tokio")))]
pub async fn wait_for<R, E, F, G>(factory: F) -> Result<R, SituwaitionError<E>>
where
    R: Send + Sync + 'static,
    E: Error + Send + Sync + 'static,
    F: Fn() -> G + Send,
    G: Future<Output = Result<R, E>> + Send,
{
    AsyncWaiter::from_factory(factory).exec().await
}

#[cfg(test)]
mod tests {
    use std::io::ErrorKind;
    use tokio::time::Duration;

    use super::*;

    #[tokio::test]
    async fn test_unit_tokio_wait_for_fn() {
        assert!(
            matches!(
                wait_for(|| async { Ok::<bool, std::io::Error>(true) }).await,
                Ok(true)
            ),
            "wait_for_fn with a simple fn is true"
        );
    }

    #[tokio::test]
    async fn test_unit_tokio_from_fn() {
        assert!(
            matches!(
                AsyncWaiter::from_factory(|| async { Ok::<bool, std::io::Error>(true) })
                    .exec()
                    .await,
                Ok(true)
            ),
            "wait_for_fn with a simple fn is true"
        );
    }

    #[tokio::test]
    async fn test_unit_tokio_exec_fail() {
        assert!(matches!(
            AsyncWaiter::with_timeout(
                || async {
                    Err::<(), std::io::Error>(std::io::Error::new(ErrorKind::Other, "test"))
                },
                Duration::from_millis(500)
            )
            .expect("failed to create")
            .exec()
            .await,
            Err(SituwaitionError::TimeoutError(std::io::Error { .. })),
        ),);
    }

    #[tokio::test]
    async fn test_unit_tokio_exec_pass() {
        assert!(
            matches!(
                AsyncWaiter::with_check_interval(
                    || async { Ok::<bool, std::io::Error>(true) },
                    Duration::from_millis(100),
                )
                .expect("failed to create")
                .exec()
                .await,
                Ok(true)
            ),
            "always passing check passes in 100m with check interval of 100ms"
        );
    }

    #[tokio::test]
    async fn test_unit_tokio_wait_for_with_timeout() {
        let start = tokio::time::Instant::now();

        assert!(
            matches!(
                AsyncWaiter::with_timeout(
                    || async {
                        Err::<(), std::io::Error>(std::io::Error::new(ErrorKind::Other, "test"))
                    },
                    Duration::from_millis(500),
                )
                .expect("failed to create")
                .exec()
                .await,
                Err(SituwaitionError::TimeoutError(std::io::Error { .. })),
            ),
            "always erroring check fails"
        );
        assert!(
            Instant::now() - start >= Duration::from_millis(500),
            "failing check waited until after timeout"
        );
    }

    #[tokio::test]
    async fn test_unit_tokio_with_check_interval() {
        let start = Instant::now();

        assert!(
            matches!(
                AsyncWaiter::with_check_interval(
                    || async { Ok::<bool, std::io::Error>(true) },
                    Duration::from_millis(100)
                )
                .expect("failed to create")
                .exec()
                .await,
                Ok(true)
            ),
            "always passing check passed"
        );
        assert!(
            Instant::now() - start < Duration::from_millis(250),
            "passed faster than default interval (250ms) w/ shorter interval"
        );
    }

    #[tokio::test]
    async fn test_unit_tokio_forever_waits_past_default_timeout() {
        // Becomes true only after longer than the default timeout (3s),
        // which would fail with a timeout if `Forever` were not honored.
        let start = Instant::now();
        let ready_at = start + Duration::from_millis(3_500);
        assert!(
            matches!(
                AsyncWaiter::with_timeout(
                    move || async move {
                        if Instant::now() >= ready_at {
                            Ok(true)
                        } else {
                            Err(std::io::Error::new(ErrorKind::Other, "not yet"))
                        }
                    },
                    Timeout::Forever
                )
                .expect("failed to create")
                .exec()
                .await,
                Ok(true)
            ),
            "check that becomes true after the default timeout succeeds with Timeout::Forever"
        );
        assert!(
            Instant::now() - start >= Duration::from_millis(3_500),
            "returned before the condition became true"
        );
    }

    #[tokio::test]
    async fn test_unit_tokio_duration_still_times_out() {
        // Same condition as the forever test, but a duration timeout must still fire.
        let ready_at = Instant::now() + Duration::from_millis(3_500);
        assert!(
            matches!(
                AsyncWaiter::with_timeout(
                    move || async move {
                        if Instant::now() >= ready_at {
                            Ok(true)
                        } else {
                            Err(std::io::Error::new(ErrorKind::Other, "not yet"))
                        }
                    },
                    Timeout::After(Duration::from_millis(500))
                )
                .expect("failed to create")
                .exec()
                .await,
                Err(SituwaitionError::TimeoutError(std::io::Error { .. })),
            ),
            "a duration timeout still times out"
        );
    }

    #[tokio::test]
    async fn test_unit_tokio_with_long_check() {
        let start = Instant::now();
        assert!(
            matches!(
                AsyncWaiter::with_timeout(
                    || async {
                        sleep(Duration::from_millis(500)).await;
                        Ok::<bool, std::io::Error>(true)
                    },
                    Duration::from_millis(250)
                )
                .expect("failed to create")
                .exec()
                .await,
                Err(SituwaitionError::CheckTimeoutError),
            ),
            "check that finishes in 500ms times out in 100ms as configured"
        );
        assert!(
            Instant::now() - start < Duration::from_millis(500),
            "timed out before the check completed"
        );
    }
}
