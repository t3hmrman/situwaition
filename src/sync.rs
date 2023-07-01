use std::{error::Error, thread::sleep, time::Instant};

use crate::{SituwaitionError, SituwaitionOpts, SyncSituwaition};

/// Synchronous situwaitioner
#[allow(dead_code)]
struct SyncExecutor {
    pub(crate) opts: SituwaitionOpts,
}

// TODO: write impls for fns that naturally should be awaitable
// - Functions, obviously
// - Closures? (they're un-nameable though...?)
// - Mutex (wait for a specific value)?
// - RwLock (wait for a speicfic value)?

// IDEA?: The wait for function could even take Sync stuff
// And would basically start polling it for completion!

/// Fully synchronous waiting for a situwaition
/// This function will sleep the main thread and pause execution!
#[allow(dead_code)]
fn wait_for<R, E>(
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
                if now - start > timeout {
                    break Err(SituwaitionError::TimeoutError(e));
                }
            }
        }

        // busy wait for the check interval
        sleep(check_interval);
    }
}
