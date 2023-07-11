use std::{
    thread::sleep,
    time::{Duration, Instant},
};

use crate::{SituwaitionBase, SituwaitionError, SituwaitionOpts, SyncSituwaition};

/// Synchronous situwaitioner
#[allow(dead_code)]
pub struct SyncWaiter<R, E> {
    /// Options for the situwaition
    opts: SituwaitionOpts,

    /// Function that can be run to decide whether the executor should finish
    check_fn: Box<dyn Fn() -> Result<R, E>>,
}

impl<R, E> SituwaitionBase for SyncWaiter<R, E> {
    type Result = R;
    type Error = E;

    fn options(&self) -> &SituwaitionOpts {
        &self.opts
    }

    fn set_options(
        &mut self,
        update_fn: impl Fn(&SituwaitionOpts) -> SituwaitionOpts,
    ) -> Result<(), SituwaitionError<()>> {
        self.opts = update_fn(&self.opts);
        Ok(())
    }
}

impl<R, E> SyncSituwaition for SyncWaiter<R, E> {
    fn exec(&self) -> Result<R, SituwaitionError<E>> {
        let start = Instant::now();

        // Run the situwaition until it succeeds
        loop {
            match (self.check_fn)() {
                Ok(v) => break Ok(v),
                Err(e) => {
                    let now = Instant::now();
                    if now - start > self.opts.timeout {
                        break Err(SituwaitionError::TimeoutError(e));
                    }
                }
            }

            // busy wait for the check interval
            sleep(self.opts.check_interval);
        }
    }
}

#[allow(dead_code)]
impl<R, E> SyncWaiter<R, E> {
    pub fn from_fn(check_fn: impl Fn() -> Result<R, E> + 'static) -> SyncWaiter<R, E> {
        SyncWaiter {
            opts: SituwaitionOpts::default(),
            check_fn: Box::new(check_fn),
        }
    }

    /// Create a sync executor with options fully specified
    pub fn with_opts(
        check_fn: impl Fn() -> Result<R, E> + 'static,
        opts: SituwaitionOpts,
    ) -> SyncWaiter<R, E> {
        SyncWaiter {
            opts,
            check_fn: Box::new(check_fn),
        }
    }

    /// Create a SyncWaiter with only timeout customized
    pub fn with_timeout(
        check_fn: impl Fn() -> Result<R, E> + 'static,
        timeout: Duration,
    ) -> SyncWaiter<R, E> {
        Self::with_opts(
            Box::new(check_fn),
            SituwaitionOpts {
                timeout,
                ..SituwaitionOpts::default()
            },
        )
    }

    /// Create a SyncWaiter with only check interval customized
    pub fn with_check_interval(
        check_fn: impl Fn() -> Result<R, E> + 'static,
        check_interval: Duration,
    ) -> SyncWaiter<R, E> {
        Self::with_opts(
            Box::new(check_fn),
            SituwaitionOpts {
                check_interval,
                ..SituwaitionOpts::default()
            },
        )
    }
}

/////////////////////
// Implementations //
/////////////////////

/// Wait for a given function to resolve with a given result.
///
/// Returning a result (as opposed to the error) will end waiting, otherwise
/// the function will be retried up until the default timeout (see SituwaitionOpts)
#[allow(dead_code)]
pub fn wait_for<R, E>(
    check_fn: impl Fn() -> Result<R, E> + 'static,
) -> Result<R, SituwaitionError<E>>
where
    E: std::error::Error,
{
    SyncWaiter::from_fn(check_fn).exec()
}

#[cfg(all(test, not(any(feature = "async-std", feature = "tokio"))))]
mod tests {
    use std::io::ErrorKind;

    use super::*;

    #[test]
    fn test_unit_wait_for_fn() {
        assert!(
            matches!(wait_for(|| Ok::<bool, std::io::Error>(true)), Ok(true)),
            "wait_for_fn with a simple fn is true"
        );
    }

    #[test]
    fn test_unit_sync_executor_from_fn() {
        assert!(
            matches!(
                SyncWaiter::from_fn(|| Ok::<bool, std::io::Error>(true)).exec(),
                Ok(true)
            ),
            "wait_for_fn with a simple fn is true"
        );
    }

    #[test]
    fn test_unit_sync_executor_exec_fail() {
        assert!(matches!(
            SyncWaiter::with_timeout(
                || Err::<(), std::io::Error>(std::io::Error::new(ErrorKind::Other, "test")),
                Duration::from_millis(500)
            )
            .exec(),
            Err(SituwaitionError::TimeoutError(std::io::Error { .. })),
        ),);
    }

    #[test]
    fn test_unit_sync_executor_exec_pass() {
        assert!(
            matches!(
                SyncWaiter::with_check_interval(
                    || Ok::<bool, std::io::Error>(true),
                    Duration::from_millis(100)
                )
                .exec(),
                Ok(true)
            ),
            "always passing check passes in 100m with check interval of 100ms"
        );
    }

    #[test]
    fn test_unit_wait_for_sync_executor_with_timeout() {
        let start = Instant::now();
        assert!(
            matches!(
                SyncWaiter::with_timeout(
                    || Err::<(), std::io::Error>(std::io::Error::new(ErrorKind::Other, "test")),
                    Duration::from_millis(500)
                ).exec(),
                Err(SituwaitionError::TimeoutError(std::io::Error { .. })),
            ),
            "always erroring check fails in 100ms with timeout of 100ms"
        );
        assert!(
            Instant::now() - start >= Duration::from_millis(500),
            "failing check passed faster than timeout"
        );
    }

    #[test]
    fn test_unit_sync_executor_with_check_interval() {
        let start = Instant::now();
        assert!(
            matches!(
                SyncWaiter::with_check_interval(
                    || Ok::<bool, std::io::Error>(true),
                    Duration::from_millis(100)
                ).exec(),
                Ok(true)
            ),
            "always passing check passes in 100m with check interval of 100ms"
        );
        assert!(
            Instant::now() - start < Duration::from_millis(250),
            "passed faster than default interval"
        );
    }
}
