# Trasy

Trasy is a Rust library designed to facilitate rich error handling by integrating traced errors with backtraces. It is particularly useful in applications where detailed context and error origin tracing are critical for debugging and error resolution.

## Features

- **Traced Errors**: Integrates with `tracing_error::SpanTrace` to capture and display the context of errors.
- **Backtrace Support**: Optionally attaches backtraces to errors to provide a detailed stack trace when an error occurs.
- **Macros for Convenience**: Includes macros `error!` and `bail!` to simplify error creation and propagation.

## Installation

Add `Trasy` to your Cargo.toml:

```toml
[dependencies]
trasy = "0.1.0"  # Use the latest version
```


## Environment Setup and Instrumentation

### Enabling Backtrace

To fully utilize the backtrace functionality in Rust, you need to set the `RUST_BACKTRACE` environment variable. This can be done by running your application with the variable set to `1`:

```bash
RUST_BACKTRACE=1 cargo run
```

This setting tells Rust to capture detailed backtraces when errors occur.

### Using `#[instrument]` with Tracing

To enhance the diagnostics of your Rust applications, use the `#[instrument]` attribute from the `tracing` crate. This attribute automatically instruments your functions, recording the entry and exit of calls, and captures arguments to the functions:

```rust
use tracing::instrument;

#[instrument]
fn compute_value(x: i32, y: i32) -> i32 {
    x + y
}
```

Using `#[instrument]` provides valuable insights into function calls and can be coupled with error handling to trace error sources more effectively.

## User Outcome

Using `TrasyError`, developers can get and read both span trace and backtrace simultaneously, providing a dual-layer of error context that enhances debugging capabilities. The output when an error occurs would look something like this:

**Error Context:**

```
An error occurred:
Error Context:
   0: tracing::foo
             at src/main.rs:158
   1: tracing::bar
           with hoge="hoge"
             at src/main.rs:153
```

**Backtrace:**

```
Backtrace:
Backtrace [{ fn: "tracing::foo::{{closure}}", file: "./src/main.rs", line: 163 }, { fn: "tracing::foo", file: "./src/main.rs", line: 158 }, { fn: "tracing::bar", file: "./src/main.rs", line: 155 }, ...]
```

## Usage

### Basic Usage

To use `Trasy`, first import it along with its macros:

```rust
use trasy::{TrasyError, error, bail};
```

Create and propagate errors easily using the `error!` macro:

```rust
fn example_function() -> Result<(), TrasyError<String>> {
    let some_result = some_error_prone_operation();

    some_result.map_err(|e| error!(e))
}
```

### Using Backtrace

To attach a backtrace to your error, simply use the error in a context where the backtrace will be captured:

```rust
fn example_function() -> Result<(), TrasyError<String>> {
    let some_result = another_error_prone_operation();
    some_result.map_err(|e| bail!(e))
}
```

### Implementing for Custom Error Types

`Trasy` can wrap any error type that implements `std::fmt::Debug` and `std::fmt::Display`. Here's how you can implement it for a custom error type:

```rust
#[derive(Debug)]
enum MyError {
    Io(std::io::Error),
    Num(std::num::ParseIntError),
}

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            MyError::Io(ref err) => write!(f, "IO error: {}", err),
            MyError::Num(ref err) => write!(f, "Parse error: {}", err),
        }
    }
}

impl Error for MyError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match *self {
            MyError::Io(ref err) => Some(err),
            MyError::Num(ref err) => Some(err),
        }
    }
}
```

### Implementing with thiserror

```rust
use thiserror::Error;

use trasy::TrasyError;
use trasy::bail;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Failed to perform operation")]
    OperationError,

    #[error("IO error occurred: {0}")]
    IoError(#[from] std::io::Error),
}

trait AppErrorExt {
    fn new(error: AppError) -> Self;
}

impl AppErrorExt for TrasyError<AppError> {
    fn new(error: AppError) -> Self {
        TrasyError::new(error)
    }
}

fn might_fail(flag: bool) -> Result<(), TrasyError<AppError>> {
    if flag {
        bail!(AppError::OperationError)
    } else {
        Ok(())
    }
}

fn main() {
    match might_fail(true) {
        Ok(_) => println!("Success!"),
        Err(e) => println!("Error: {}", e),
    }
}
```

## Contributing

Contributions to `Trasy` are welcome! Here are some ways you can contribute:

- Reporting bugs
- Suggesting enhancements
- Adding more integrations and features
- Improving documentation

Please feel free to fork the repository and submit pull requests.
