//! Situwaition
//!
//! <hr>
//!
//! This library makes it easy to wait for a situation (cutely named "situWAITion") to complete.
//!
//! Situwaition can be used in contexts with or without async runtimes, and generally does what you'd expect (tm):
//!
//! ```
//! use situwaition::wait_for;
//!
//! fn main() -> Result<(), Box<dyn Error>> {
//!     let value = 0;
//!
//!     let result: Result<&str> = wait_for(|| match value == 5 {
//!         true => Ok("done!"),
//!         false => {
//!             value += 1; // NOTE: incrementing like this only works in a single threaded context!
//!             Err("not yet")
//!         },
//!     });
//! }
//! ```

use std::{result::Result, time::Duration};

use derive_builder::Builder;
use thiserror::Error;

#[cfg(any(feature = "tokio", feature = "async-std"))]
use async_trait::async_trait;

pub mod runtime;
pub mod sync;

const DEFAULT_SITUWAITION_TIMEOUT_MS: u64 = 3_000;
const DEFAULT_SITUWAITION_CHECK_INTERVAL_MS: u64 = 250;

pub use sync::wait_for;

/// The type of error that is thrown when
#[derive(Debug, Error)]
pub enum SituwaitionError<E> {
    /// Timeout from repeated failure
    #[error("failed repeatedly until the timeout: {0}")]
    TimeoutError(E),

    /// A single conditoin failure
    #[error("condition check failed: {0}")]
    ConditionFailed(E),

    #[cfg(feature = "tokio")]
    #[error("error joining tokio task: {0}")]
    TokioJoinError(tokio::task::JoinError),

    #[cfg(feature = "tokio")]
    #[error("unexpected error")]
    UnexpectedError,
}

/// Options for a given situwaition
#[allow(dead_code)]
#[derive(Debug, Clone, Builder)]
pub struct SituwaitionOpts {
    pub timeout: Duration,
    pub check_interval: Duration,
}

impl Default for SituwaitionOpts {
    fn default() -> Self {
        SituwaitionOpts {
            timeout: Duration::from_millis(DEFAULT_SITUWAITION_TIMEOUT_MS),
            check_interval: Duration::from_millis(DEFAULT_SITUWAITION_CHECK_INTERVAL_MS),
        }
    }
}

/// The basic requirements of any situwaition
trait SituwaitionBase {
    type Result;
    type Error;

    /// Retrieve the options associated with this situwaition
    fn options(&self) -> &SituwaitionOpts;

    /// Change the options associated with this situwaition
    fn set_options(
        &mut self,
        update_fn: impl Fn(&SituwaitionOpts) -> SituwaitionOpts,
    ) -> Result<(), SituwaitionError<()>>;
}

/// Synchronously executed situwaitions
trait SyncSituwaition: SituwaitionBase {
    /// Execute the situwaition, and wait until it resolves
    /// or fails with a timeout
    fn exec(&self) -> Result<Self::Result, Self::Error>;
}

/// This trait represents a "situwaition" that can be a"waited".
/// note that how the waiting is done can differ by platform
#[cfg(any(feature = "tokio", feature = "async-std"))]
#[async_trait]
trait AsyncSituwaition: SituwaitionBase {
    /// Execute the situwaition, and wait until it resolves
    /// or fails with a timeout
    async fn exec(&mut self) -> Result<Self::Result, SituwaitionError<Self::Error>>;
}

// It's possible that Situation-as-object is superior...
//
// (!) MAYBE do both? taking the impl is the most permissive way to allow it to work
// AND we can provide special interfaces for specific things.
//
// Does situation-as-object enable sync/async use easier? fn -> Result<Future<...>, Error> ??
// Do the simple thing and just spawn tasks to do the work synchronously but far away?
// (!) NO, if async is enabled, then exec should be an async function!

// Now can the run function be hidden?

// Q: Should people be able to manipulate the waiting stuff *at all*?
// maybe they should just *get back* something that adheres to the interface?
//
// kind of like a 'zero trust' interface -- fn (in: impl A) -> impl B

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn it_works() {
//         let result = add(2, 2);
//         assert_eq!(result, 4);
//     }
// }
