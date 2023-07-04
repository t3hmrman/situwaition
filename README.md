# ⏲ `situwation` - wait for conditions happen

`situwaition` is an utility library for waiting.

## Install

To use `situwaition` in your rust project

```console
cargo add situwaition
```

## Quickstart

From your code:

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

## Supported environments

`situwaition` works with the following environments:

| Name                              | Supported? |
|-----------------------------------|------------|
| Synchronous                       | ✅         |
| Async w/ [`tokio`][tokio]         | 🛠          |
| Async w/ [`async-std`][async-std] | 🛠          |

[tokio]: https://crates.io/crates/tokio
[async-std]: https://crates.io/crates/async-std
