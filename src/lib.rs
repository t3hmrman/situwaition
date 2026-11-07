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
//!
//! The example above demonstrates the synchronous usage, but `tokio` and `async-std` and corresponding modules are available as well.
#![cfg_attr(all(doc, CHANNEL_NIGHTLY), feature(doc_cfg))]

use std::{result::Result, time::Duration};

use derive_builder::Builder;
use thiserror::Error;

#[cfg(any(feature = "tokio", feature = "async-std"))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "async-std", feature = "tokio"))))]
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

/// How long a situwaition should be waited on
///
/// A [`Duration`] converts into [`Timeout::After`], so anywhere a `Timeout` is accepted
/// you can pass a `Duration` (via `.into()` or, for `impl Into<Timeout>` parameters, directly).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Timeout {
    /// Wait forever, i.e. until the check succeeds.
    ///
    /// Note that a check that never succeeds will make the situwaition never return.
    Forever,

    /// Give up after the given amount of time
    After(Duration),
}

impl Timeout {
    /// The duration to wait for, or `None` if waiting forever
    pub fn as_duration(&self) -> Option<Duration> {
        match self {
            Timeout::Forever => None,
            Timeout::After(d) => Some(*d),
        }
    }
}

impl From<Duration> for Timeout {
    fn from(d: Duration) -> Self {
        Timeout::After(d)
    }
}

/// Options for a given situwaition
#[allow(dead_code)]
#[derive(Debug, Clone, Builder)]
pub struct SituwaitionOpts {
    /// The maximum time to wait for a situwaition (or [`Timeout::Forever`])
    pub timeout: Timeout,

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
            timeout: Timeout::After(Duration::from_millis(DEFAULT_SITUWAITION_TIMEOUT_MS)),
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

/// This trait represents a "situwaition" that can be a"waited", with tokio.
/// note that how the waiting is done can differ by platform
#[cfg(feature = "tokio")]
#[cfg_attr(docsrs, doc(cfg(feature = "tokio")))]
#[async_trait]
pub trait TokioAsyncSituwaition: SituwaitionBase {
    /// Execute the situwaition, and wait until it resolves
    /// or fails with a timeout
    async fn exec(&mut self) -> Result<Self::Result, SituwaitionError<Self::Error>>;
}

/// This trait represents a "situwaition" that can be a"waited", with async-std.
/// note that how the waiting is done can differ by platform
#[cfg(feature = "async-std")]
#[cfg_attr(docsrs, doc(cfg(feature = "async-std")))]
#[async_trait]
pub trait AsyncStdAsyncSituwaition: SituwaitionBase {
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
