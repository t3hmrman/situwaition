use std::{
    result::Result,
    sync::{Arc, Mutex},
};

use situwaition::sync::wait_for;
use thiserror::Error;

#[derive(Debug, Error)]
enum ExampleError {
    #[error("not done counting yet")]
    NotDoneCountingError,

    #[error("mutex encounted a poison error")]
    MutexPoisonError,
}

// This example uses wait_for to wait for a value that changes, completely synchronously.
//
// By default, wait_for checks every 250ms, and times out after 3 seconds.
// this means the code below should wait 750ms in total, and value should never be above 3.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let value = Arc::new(Mutex::new(0));
    let shared_value = value.clone();

    let result = wait_for(move || {
        // Get the current value from the mutex
        let mut current_value = shared_value
            .lock()
            .map_err(|_| ExampleError::MutexPoisonError)?;

        // Act on the current value
        if *current_value >= 3 {
            Ok(42)
        } else {
            *current_value += 1;
            Err(ExampleError::NotDoneCountingError)
        }
    })?;

    assert!(matches!(result, 42));
    eprintln!("resulting value is: {}", result);

    Ok(())
}
