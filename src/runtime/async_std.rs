#![cfg(feature = "async-std")]

use std::{error::Error, future::Future, time::Instant};

use async_std::{future::timeout, task::sleep};
use async_trait::async_trait;

use crate::{AsyncStdAsyncSituwaition, SituwaitionError};

use super::AsyncWaiter;

#[async_trait]
#[cfg_attr(docsrs, doc(cfg(feature = "async-std")))]
impl<F, A, R, E> AsyncStdAsyncSituwaition for AsyncWaiter<F, A, R, E>
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
            match timeout(check_timeout, fut).await {
                // Check completed in time and successfully and we can return
                Ok(Ok(v)) => return Ok(v),
                // Check timed out
                Err(_) => return Err(SituwaitionError::CheckTimeoutError),
                // Check completed in time but failed
                Ok(Err(e)) => {
                    if let Some(t) = cooldown {
                        sleep(t).await;
                    }

                    if Instant::now() - start > self.opts.timeout {
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
/// Returning a [Result::Ok] will end waiting, and [Result::Err]s will be ignored.
/// The function produced by teh factory will be retried up until the default timeout (see [SituwaitionOpts][crate::SituwaitionOpts])
#[allow(dead_code)]
#[cfg_attr(docsrs, doc(cfg(feature = "async-std")))]
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
    use std::{io::ErrorKind, time::Duration};

    use super::*;

    #[async_std::test]
    async fn test_unit_async_std_wait_for_fn() {
        assert!(
            matches!(
                wait_for(|| async { Ok::<bool, std::io::Error>(true) }).await,
                Ok(true)
            ),
            "wait_for_fn with a simple fn is true"
        );
    }

    #[async_std::test]
    async fn test_unit_async_std_from_fn() {
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

    #[async_std::test]
    async fn test_unit_async_std_exec_fail() {
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

    #[async_std::test]
    async fn test_unit_async_std_exec_pass() {
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

    #[async_std::test]
    async fn test_unit_async_std_wait_for_with_timeout() {
        let start = Instant::now();

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

    #[async_std::test]
    async fn test_unit_async_std_with_check_interval() {
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

    #[async_std::test]
    async fn test_unit_async_std_with_long_check() {
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
