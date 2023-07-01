///! Situwaition
///!
///! <hr>
///!
///! This library makes it easy to wait for a situation (cutely named "situWAITion") to complete.
///!
///! Situwaition can be used in contexts with or without async runtimes, and generally does what you'd expect (tm):
///!
///! ```
///! use situwaition::wait_for;
///!
///! fn main() -> Result<(), Box<dyn Error>> {
///!     let value = 0;
///!
///!     let result: Result<&str> = wait_for(|| match value == 5 {
///!         true => Ok("done!"),
///!         false => {
///!             value += 1; // NOTE: incrementing like this only works in a single threaded context!
///!             Err("not yet")
///!         },
///!     });
///! }
///! ```
use std::{
    error::Error,
    result::Result,
    thread::sleep,
    time::{Duration, Instant},
};

use thiserror::Error;

/// The type of error that is thrown when
#[derive(Debug, Error)]
pub enum SituwaitionError<E> {
    /// Errors that can arise from a situwation
    #[error("update failed: {0}")]
    UpdateFailed(String),

    /// Errors from repeated failure
    #[error("failed repeatedly until the timeout: {0}")]
    TimeoutError(E),
}

/// Options for a given situwaition
#[allow(dead_code)]
struct SituwaitionOpts {
    pub timeout: Duration,
    pub check_interval: Duration,
}

/// This trait represents a "situwaition" that can be a"waited".
/// note that how the waiting is done can differ by platform
trait Situwaition {
    type Result;
    type Error;

    /// Retrieve the options associated with this situwaition
    fn options(&self) -> SituwaitionOpts;

    /// Change the options associated with this situwaition
    fn set_options(
        &mut self,
        update_fn: dyn Fn(SituwaitionOpts) -> SituwaitionOpts,
    ) -> Result<SituwaitionOpts, SituwaitionError<()>>;

    /// Execute the situwaition, and wait until it resolves
    /// or fails with a timeout
    fn exec(&self) -> Result<Self::Result, Self::Error>;
}

/// Wait for a situwaition
#[allow(dead_code)]
fn wait_for<R, E>(
    wait_fn: impl Situwaition<Result = R, Error = E>,
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

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn it_works() {
//         let result = add(2, 2);
//         assert_eq!(result, 4);
//     }
// }
