use std::{error::Error, future::Future, time::Duration};

use derive_builder::Builder;

use crate::{SituwaitionBase, SituwaitionError, SituwaitionOpts};

#[cfg(feature = "async-std")]
mod async_std;
#[cfg(feature = "tokio")]
mod tokio;

#[derive(Builder)]
pub struct AsyncSituwaiter<F, A, R, E>
where
    F: Future<Output = Result<R, E>> + Send,
    A: Fn() -> F + Send,
    R: Send + Sync,
    E: Error + Send + Sync,
{
    /// Options for the situwaition
    pub opts: SituwaitionOpts,

    /// The async task that should be used to check completion
    pub factory: A,
}

impl<F, A, R, E> SituwaitionBase for AsyncSituwaiter<F, A, R, E>
where
    F: Future<Output = Result<R, E>> + Send,
    A: Fn() -> F + Send,
    R: Send + Sync,
    E: Error + Send + Sync,
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

impl<F, A, R, E> AsyncSituwaiter<F, A, R, E>
where
    F: Future<Output = Result<R, E>> + Send,
    A: Fn() -> F + Send,
    R: Send + Sync,
    E: Error + Send + Sync,
{
    /// Convert an existing async function factory into an AsyncSituwaiter
    #[allow(dead_code)]
    fn from_factory(factory: A) -> AsyncSituwaiter<F, A, R, E> {
        AsyncSituwaiter {
            opts: SituwaitionOpts::default(),
            factory,
        }
    }

    /// Create a sync executor with options fully specified
    #[allow(dead_code)]
    fn with_opts(factory: A, opts: SituwaitionOpts) -> AsyncSituwaiter<F, A, R, E> {
        AsyncSituwaiter { opts, factory }
    }

    /// Create a SyncExecutor with only timeout customized
    #[allow(dead_code)]
    fn with_timeout(factory: A, timeout: Duration) -> AsyncSituwaiter<F, A, R, E> {
        Self::with_opts(
            factory,
            SituwaitionOpts {
                timeout,
                ..SituwaitionOpts::default()
            },
        )
    }

    /// Create a SyncExecutor with only check interval customized
    #[allow(dead_code)]
    fn with_check_interval(factory: A, check_interval: Duration) -> AsyncSituwaiter<F, A, R, E> {
        Self::with_opts(
            factory,
            SituwaitionOpts {
                check_interval,
                ..SituwaitionOpts::default()
            },
        )
    }
}
