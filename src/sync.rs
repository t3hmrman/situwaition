use std::{
    error::Error,
    thread::sleep,
    time::{Duration, Instant},
};

use crate::{SituwaitionBase, SituwaitionError, SituwaitionOpts, SyncSituwaition};

/// Synchronous situwaitioner
#[allow(dead_code)]
struct SyncWaiter<R, E> {
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
    fn exec(&self) -> Result<R, E> {
        (self.check_fn)()
    }
}

#[allow(dead_code)]
impl<R, E> SyncWaiter<R, E> {
    fn from_fn(check_fn: impl Fn() -> Result<R, E> + 'static) -> SyncWaiter<R, E> {
        SyncWaiter {
            opts: SituwaitionOpts::default(),
            check_fn: Box::new(check_fn),
        }
    }

    /// Create a sync executor with options fully specified
    fn with_opts(
        check_fn: impl Fn() -> Result<R, E> + 'static,
        opts: SituwaitionOpts,
    ) -> SyncWaiter<R, E> {
        SyncWaiter {
            opts,
            check_fn: Box::new(check_fn),
        }
    }

    /// Create a SyncWaiter with only timeout customized
    fn with_timeout(
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
    fn with_check_interval(
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

// TODO: write impls for fns that naturally should be awaitable
// - Functions, obviously
// - Closures? (they're un-nameable though...?)
// - Mutex (wait for a specific value)?
// - RwLock (wait for a speicfic value)?

// TODO: Builder w/ https://crates.io/crates/derive_builder

// IDEA?: The wait for function could even take Sync stuff
// And would basically start polling it for completion!

/// Fully synchronous waiting for a situwaition
/// This function will sleep the main thread and pause execution!
#[allow(dead_code)]
fn _wait_for<R, E>(
    wait_fn: impl SyncSituwaition<Result = R, Error = E>,
) -> Result<R, SituwaitionError<E>>
where
    E: Error,
{
    let SituwaitionOpts {
        timeout,
        check_interval,
    } = wait_fn.options();
    let start = Instant::now();

    // Run the situwaition until it succeeds
    loop {
        match wait_fn.exec() {
            Ok(v) => break Ok(v),
            Err(e) => {
                let now = Instant::now();
                if now - start > *timeout {
                    break Err(SituwaitionError::TimeoutError(e));
                }
            }
        }

        // busy wait for the check interval
        sleep(*check_interval);
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
    _wait_for(SyncWaiter::from_fn(check_fn))
}

#[cfg(test)]
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
            Err(std::io::Error { .. }),
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
                _wait_for(SyncWaiter::with_timeout(
                    || Err::<(), std::io::Error>(std::io::Error::new(ErrorKind::Other, "test")),
                    Duration::from_millis(500)
                )),
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
                _wait_for(SyncWaiter::with_check_interval(
                    || Ok::<bool, std::io::Error>(true),
                    Duration::from_millis(100)
                )),
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
