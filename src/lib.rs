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

    #[error("check fn run exceeded the timeout")]
    CheckTimeoutError,

    /// A single conditoin failure
    #[error("condition check failed: {0}")]
    ConditionFailed(E),

    #[error("unexpected error: {0}")]
    UnexpectedError(String),
}

/// Options for a given situwaition
#[allow(dead_code)]
#[derive(Debug, Clone, Builder)]
pub struct SituwaitionOpts {
    /// The maximum time to wait for a situwaition
    pub timeout: Duration,

    /// How often to check for a passing condition.
    /// Note that in the synchronous case, this determines how quickly
    /// you can return *before* a check actually completes (i.e. timing in 100ms when check_fn takes 500ms)
    pub check_interval: Duration,

    /// Time to wait after a check has been performed.
    /// Use this to avoid running resource-intensive checks too frequently
    pub check_cooldown: Option<Duration>,
}

impl Default for SituwaitionOpts {
    fn default() -> Self {
        SituwaitionOpts {
            timeout: Duration::from_millis(DEFAULT_SITUWAITION_TIMEOUT_MS),
            check_interval: Duration::from_millis(DEFAULT_SITUWAITION_CHECK_INTERVAL_MS),
            check_cooldown: None,
        }
    }
}

/// The basic requirements of any situwaition
pub trait SituwaitionBase {
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
pub trait SyncSituwaition: SituwaitionBase {
    /// Execute the situwaition, and wait until it resolves
    /// or fails with a timeout
    fn exec(&mut self) -> Result<Self::Result, SituwaitionError<Self::Error>>;
}

/// This trait represents a "situwaition" that can be a"waited".
/// note that how the waiting is done can differ by platform
#[cfg(any(feature = "tokio", feature = "async-std"))]
#[async_trait]
pub trait AsyncSituwaition: SituwaitionBase {
    /// Execute the situwaition, and wait until it resolves
    /// or fails with a timeout
    async fn exec(&mut self) -> Result<Self::Result, SituwaitionError<Self::Error>>;
}

/// Errors that are thrown during waiter creation
#[derive(Debug, Clone, Error)]
pub enum WaiterCreationError {
    #[error("invalid timeout: {0}")]
    InvalidTimeout(String),

    #[error("invalid interval: {0}")]
    InvalidInterval(String),
}
