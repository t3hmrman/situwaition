<h1 align="center">⏲ <code>situwaition</code></h1>

`situwaition` runs a closure *continuously*, until an `Ok(..)` is received, or a timeout period elapses.

## Install

```console
cargo add situwaition                      # only sync waiting is enabled by default
cargo add situwaition --features async-std # use async-std
cargo add situwaition --features tokio     # use tokio
```

If you're editing `Cargo.toml` by hand:

```toml
[dependencies]
situwaition = "0.1"
#situwaition = { version = "0.1", features = [ "async-std" ] }
#situwaition = { version = "0.1", features = [ "tokio" ] }
```

> **Warning**
> `situation` does not allow using both `async-std` and `tokio` features at the same time (see [the FAQ below](#FAQ)).

## Quickstart

### Sync

To use `situwaition` in synchronous contexts:

```rust
use situwaition::wait_for;

// ...

    // Do some waiting
    let result = wait_for(move || {
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
- The configured timeout (3s by default) is reached.

See a full example in [`examples/basic_sync.rs`](./examples/basic_sync.rs).

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

See a full example in [`examples/tokio/main.rs`](./examples/tokio/main.rs).

### async-std

If you're using [`async-std`][async-std], then your code looks like this:

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

See a full example in [`examples/async-std/main.rs`](./examples/async-std/main.rs).

## Supported environments

`situwaition` works with the following environments:

| Name                              | Supported? |
|-----------------------------------|------------|
| Synchronous                       | ✅         |
| Async w/ [`tokio`][tokio]         | ✅         |
| Async w/ [`async-std`][async-std] | ✅         |

[tokio]: https://crates.io/crates/tokio
[async-std]: https://crates.io/crates/async-std

## FAQ

### Why does `situwaition` assume that I'm using *either* `async-std` or `tokio`

Because you probably are. If this is a problem for you, it *can* be changed, file an issue and let's chat about it.

## Development

To get started working on developing `situwatiion`, run the following [`just`][just] targets:

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
