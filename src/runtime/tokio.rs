#![cfg(feature = "tokio")]

use std::{error::Error, future::Future};

use async_trait::async_trait;
use tokio::time::{sleep, Instant};

use crate::{AsyncSituwaition, SituwaitionError};

use super::AsyncSituwaiter;

#[async_trait]
impl<F, A, R, E> AsyncSituwaition for AsyncSituwaiter<F, A, R, E>
where
    F: Future<Output = Result<R, E>> + Send,
    A: Fn() -> F + Send,
    R: Send + Sync,
    E: Error + Send + Sync,
{
    async fn exec(&mut self) -> Result<R, SituwaitionError<E>> {
        let start = Instant::now();

        loop {
            let fut = (self.factory)();
            match fut.await {
                Ok(v) => return Ok(v),
                Err(e) => {
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
/// Returning a result (as opposed to the error) will end waiting, otherwise
/// the function will be retried up until the default timeout (see SituwaitionOpts)
#[allow(dead_code)]
pub async fn wait_for<R, E, F, G>(factory: F) -> Result<R, SituwaitionError<E>>
where
    R: Send + Sync + 'static,
    E: Error + Send + Sync + 'static,
    F: Fn() -> G + Send,
    G: Future<Output = Result<R, E>> + Send,
{
    AsyncSituwaiter::from_factory(factory).exec().await
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
    async fn test_unit_tokio_async_executor_from_fn() {
        assert!(
            matches!(
                AsyncSituwaiter::from_factory(|| async { Ok::<bool, std::io::Error>(true) })
                    .exec()
                    .await,
                Ok(true)
            ),
            "wait_for_fn with a simple fn is true"
        );
    }

    #[tokio::test]
    async fn test_unit_tokio_async_executor_exec_fail() {
        assert!(matches!(
            AsyncSituwaiter::with_timeout(
                || async {
                    Err::<(), std::io::Error>(std::io::Error::new(ErrorKind::Other, "test"))
                },
                Duration::from_millis(500)
            )
            .exec()
            .await,
            Err(SituwaitionError::TimeoutError(std::io::Error { .. })),
        ),);
    }

    #[tokio::test]
    async fn test_unit_tokio_async_executor_exec_pass() {
        assert!(
            matches!(
                AsyncSituwaiter::with_check_interval(
                    || async { Ok::<bool, std::io::Error>(true) },
                    Duration::from_millis(100),
                )
                .exec()
                .await,
                Ok(true)
            ),
            "always passing check passes in 100m with check interval of 100ms"
        );
    }

    #[tokio::test]
    async fn test_unit_tokio_wait_for_async_executor_with_timeout() {
        let start = tokio::time::Instant::now();

        assert!(
            matches!(
                AsyncSituwaiter::with_timeout(
                    || async {
                        Err::<(), std::io::Error>(std::io::Error::new(ErrorKind::Other, "test"))
                    },
                    Duration::from_millis(500),
                )
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
    async fn test_unit_tokio_async_executor_with_check_interval() {
        let start = Instant::now();

        assert!(
            matches!(
                AsyncSituwaiter::with_check_interval(
                    || async { Ok::<bool, std::io::Error>(true) },
                    Duration::from_millis(100)
                )
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
}
