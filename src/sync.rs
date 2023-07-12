use std::{
    sync::{Arc, Mutex},
    thread::sleep,
    time::{Duration, Instant},
};

use crate::{
    SituwaitionBase, SituwaitionError, SituwaitionOpts, SyncSituwaition, WaiterCreationError,
    DEFAULT_SITUWAITION_CHECK_INTERVAL_MS, DEFAULT_SITUWAITION_TIMEOUT_MS,
};

/// Synchronous situwaitioner
#[allow(dead_code)]
pub struct SyncWaiter<R, E, F>
where
    R: Send,
    E: Send + 'static,
    F: Fn() -> Result<R, E> + Send + Sync + 'static,
{
    /// Options for the situwaition
    opts: SituwaitionOpts,

    /// Function that can be run to decide whether the executor should finish
    check_fn: Option<Box<F>>,
}

impl<R, E, F> SituwaitionBase for SyncWaiter<R, E, F>
where
    R: Send,
    E: Send + 'static,
    F: Fn() -> Result<R, E> + Send + Sync + 'static,
{
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

impl<R, E, F> SyncSituwaition for SyncWaiter<R, E, F>
where
    R: Send + 'static,
    E: Send + 'static,
    F: Fn() -> Result<R, E> + Send + Sync + 'static,
{
    fn exec(&mut self) -> Result<R, SituwaitionError<E>> {
        let start = Instant::now();

        let check_fn = self
            .check_fn
            .take()
            .ok_or_else(|| SituwaitionError::UnexpectedError("no check fn specified".into()))?;

        let result = Mutex::new(None);
        let result_read = Arc::new(result);
        let result_write = result_read.clone();

        let err: Mutex<Option<SituwaitionError<E>>> = Mutex::new(None);
        let err_read = Arc::new(err);
        let err_write = err_read.clone();

        let timeout = self.opts.timeout;
        let check_cooldown = self.opts.check_cooldown;

        // We run the check function in a scoped thread in order to ensure
        // that we can handle the case where the check function never returns in time
        std::thread::spawn(move || loop {
            let res = check_fn();
            match res {
                Ok(v) => {
                    let mut mutex = result_write.lock().map_err(|_| ())?;
                    *mutex = Some(v);
                    break Ok::<(), ()>(());
                }
                Err(e) => {
                    let now = Instant::now();
                    if now - start > timeout {
                        let mut mutex = err_write.lock().map_err(|_| ())?;
                        *mutex = Some(SituwaitionError::TimeoutError(e));
                        break Ok::<(), ()>(());
                    }

                    // If a cooldown was specified, wait a bit
                    if let Some(t) = check_cooldown {
                        sleep(t);
                    }
                }
            }
        });

        // Wait until the handle is finished, without being blocked by it
        loop {
            // Sleep & check for timeout until finished
            match (result_read.lock(), err_read.lock()) {
                (Ok(mut r), Ok(mut err)) => {
                    // If an error has returned, it *must* be the timeout error, return that quickly
                    if err.is_some() {
                        return Err((*err).take().unwrap());
                    }

                    // If we've timed out otherwise while doing the check, we timed out *during* a check
                    if Instant::now() - start > self.opts.timeout {
                        return Err(SituwaitionError::CheckTimeoutError);
                    }

                    // If a result came through, we can return that happily
                    if r.is_some() {
                        return Ok((*r).take().unwrap());
                    }
                }
                _ => panic!("failed to lock mutex"),
            }
            // Sleep before checking again
            sleep(self.opts.check_interval);
        }
    }
}

#[allow(dead_code)]
impl<R, E, F> SyncWaiter<R, E, F>
where
    R: Send + 'static,
    E: Send + 'static,
    F: Fn() -> Result<R, E> + Send + Sync + 'static,
{
    pub fn from_fn(check_fn: F) -> Self {
        SyncWaiter {
            opts: SituwaitionOpts::default(),
            check_fn: Some(Box::new(check_fn)),
        }
    }

    /// Create a sync executor with options fully specified
    pub fn with_opts(check_fn: F, opts: SituwaitionOpts) -> Self {
        SyncWaiter {
            opts,
            check_fn: Some(Box::new(check_fn)),
        }
    }

    /// Create a SyncWaiter with only timeout customized
    pub fn with_timeout(check_fn: F, timeout: Duration) -> Result<Self, WaiterCreationError> {
        if timeout < Duration::from_millis(DEFAULT_SITUWAITION_CHECK_INTERVAL_MS) {
            return Err(WaiterCreationError::InvalidTimeout(
                format!("supplied timeout ({}ms) is shorter the default timeout ({DEFAULT_SITUWAITION_CHECK_INTERVAL_MS}ms)", timeout.as_millis())
            ));
        }
        Ok(Self::with_opts(
            check_fn,
            SituwaitionOpts {
                timeout,
                ..SituwaitionOpts::default()
            },
        ))
    }

    /// Create a SyncWaiter with only check interval customized
    pub fn with_check_interval(
        check_fn: F,
        check_interval: Duration,
    ) -> Result<Self, WaiterCreationError> {
        if check_interval > Duration::from_millis(DEFAULT_SITUWAITION_TIMEOUT_MS) {
            return Err(WaiterCreationError::InvalidTimeout(
                format!("supplied check interval ({}ms) is larger than the default timeout ({DEFAULT_SITUWAITION_TIMEOUT_MS}ms)", check_interval.as_millis())
            ));
        }
        Ok(Self::with_opts(
            check_fn,
            SituwaitionOpts {
                check_interval,
                ..SituwaitionOpts::default()
            },
        ))
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
pub fn wait_for<R, E, F>(check_fn: F) -> Result<R, SituwaitionError<E>>
where
    R: Send + 'static,
    E: std::error::Error + Send + 'static,
    F: Fn() -> Result<R, E> + Send + Sync + 'static,
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
            .expect("failed to create")
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
                .expect("failed to create")
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
                )
                .expect("failed to create")
                .exec(),
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
                )
                .expect("failed to create")
                .exec(),
                Ok(true)
            ),
            "always passing check passes in 100m with check interval of 100ms"
        );
        assert!(
            Instant::now() - start < Duration::from_millis(250),
            "passed faster than default interval"
        );
    }

    #[test]
    fn test_unit_sync_executor_with_long_check() {
        let start = Instant::now();
        assert!(
            matches!(
                SyncWaiter::with_timeout(
                    || {
                        std::thread::sleep(Duration::from_millis(500));
                        Ok::<bool, std::io::Error>(true)
                    },
                    Duration::from_millis(250)
                )
                .expect("failed to create")
                .exec(),
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
