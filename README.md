<h1 align="center">⏲ <code>situwaition</code></h1>

`situwaition` runs a closure *continuously*, until an `Ok(..)` is received, or a timeout period elapses (or never, if you choose to wait forever).

## Install

```console
cargo add situwaition                      # only sync waiting is enabled by default
cargo add situwaition --features async-std # use async-std
cargo add situwaition --features tokio     # use tokio
```

If you're editing `Cargo.toml` by hand:

```toml
[dependencies]
situwaition = "0.3"
#situwaition = { version = "0.3", features = [ "async-std" ] }
#situwaition = { version = "0.3", features = [ "tokio" ] }
```

## Quickstart

### Sync

To use `situwaition` in synchronous contexts:

```rust
use situwaition::wait_for;

// ...

    // Do some waiting
    let result = wait_for(|| {
        // Get the current value from the mutex
        if some_condition { Ok(value) } else { Err(SomeError) ]
    });

    // Act on the result
    match result {
        Ok(v) => { ... }
        Err(SituwaitionError::TimeoutError(e)) => { ... }
    }

// ...
```

`situwaition` will run the function continuously, *ignoring* `Error(..)` responses until:

- The function resolves to an `Ok(..)` variant
- The configured timeout (3s by default, checking every 250ms) is reached.

See a full example in [`examples/sync.rs`](./examples/sync.rs). To run the sync example:

```
cargo run --example sync
```

### Tokio

If you're using [tokio][tokio], then your code looks like this:

```rust
use situwaition::runtime::tokio::wait_for;

// ...

    // Do some waiting
    let result = wait_for(|| async {
        // Get the current value from the mutex
        if some_condition { Ok(value) } else { Err(SomeError) ]
    });

    // Act on the result
    match result {
        Ok(v) => { ... }
        Err(SituwaitionError::TimeoutError(e)) => { ... }
    }

// ...
```

Note here that you are passing a *`Future` factory* to the function -- a function/closure (`|| { ... }`) that *outputs* a `Future` (`async { .. }`).

The usual `async` usage rules apply -- use `move`, `Arc`s, `Mutex`es, and other ownership/synchronization primitives where appropriate.

See a full example in [`examples/tokio.rs`](./examples/tokio.rs). To run the tokio example:

```console
cargo run --example tokio --features=tokio
```

### async-std

If you're using [`async-std`][async-std], then your code looks like this:

```rust
use situwaition::runtime::async_std::wait_for;

// ...

    // Do some waiting
    let result = wait_for(|| async {
        // Get the current value from the mutex
        if some_condition { Ok(value) } else { Err(SomeError) ]
    });

    // Act on the result
    match result {
        Ok(v) => { ... }
        Err(SituwaitionError::TimeoutError(e)) => { ... }
    }

// ...
```

See a full example in [`examples/async_std.rs`](./examples/async_std.rs). To run the async-std example:

```console
cargo run --example async-std --features=async-std
```

### Verbose configuration

If you'd like to control more finely the intervals and how many times a check will occur, you can create the `Waiter` object(s) yourself:

```rust
use situwaition::runtime::AsyncWaiter;
use situwaition::runtime::SyncWaiter;

// Synchronous code
SyncWaiter::with_timeout(|| { ... }, Duration::from_millis(500))?;

// Asynchronous code (either tokio or async-std)
AsyncWaiter::with_timeout(|| async { ... }, Duration::from_millis(500))?
    .exec()
    .await;
```

### Waiting forever

The timeout is a `Timeout`, which is either `Timeout::After(Duration)` or `Timeout::Forever`. A plain `Duration` converts into `Timeout::After`, so the examples above work as written. To wait with no time limit at all, pass `Timeout::Forever`:

```rust
use situwaition::Timeout;
use situwaition::runtime::AsyncWaiter;
use situwaition::sync::SyncWaiter;

// Synchronous code
SyncWaiter::with_timeout(|| { ... }, Timeout::Forever)?
    .exec();

// Asynchronous code (either tokio or async-std)
AsyncWaiter::with_timeout(|| async { ... }, Timeout::Forever)?
    .exec()
    .await;
```

Note that a check that never succeeds will make a `Timeout::Forever` situwaition wait forever. If you build `SituwaitionOpts` yourself, its `timeout` field is a `Timeout` (use `Timeout::After(duration)` or `duration.into()`).

*Upgrading from 0.3:* `SituwaitionOpts::timeout` changed from `Duration` to `Timeout`. Callers of `with_timeout` that pass a `Duration` need no changes.

See the methods on [`SyncWaiter`](./src/sync.rs) and [`AsyncWaiter`](./src/runtime/mod.rs) for more options.

## Supported environments

`situwaition` works with the following environments:

| Name                              | Supported? |
|-----------------------------------|------------|
| Synchronous                       | ✅         |
| Async w/ [`tokio`][tokio]         | ✅         |
| Async w/ [`async-std`][async-std] | ✅         |

[tokio]: https://crates.io/crates/tokio
[async-std]: https://crates.io/crates/async-std

## Development

To get started working on developing `situwaition`, run the following [`just`][just] targets:

```console
just setup build
```

To check that your changes are fine, you'll probably want to run:

```console
just test
```

If you want to see the full list of targets available that you can run `just` without any arguments.

```console
just
```

There are a few useful targets like `just build-watch` which will continuously build the project thanks to [`cargo watch`][cargo-watch].

[just]: https://github.com/casey/just
[cargo-watch]: https://crates.io/crates/cargo-watch

## Contributing

Contributions are welcome! If you find a bug or an impovement that should be included in `situwaition`, [create an issue](https://github.com/t3hmrman/situwaition/issues) or open a pull request.
