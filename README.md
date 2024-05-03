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

## OpenTelemetry Integration

`Trasy` supports OpenTelemetry, allowing you to trace your applications and export telemetry data to your chosen backend (e.g., Jaeger, Zipkin). This section describes how to configure and use OpenTelemetry in your application.

### Configuration

To use OpenTelemetry with `TrasyError`, you first need to set up the telemetry configuration. Here's how you can configure and enable telemetry:

#### 1. Define the Configuration

Configure the telemetry settings using the `TelemetryConfig` struct. You can specify the service name, the endpoint, whether to use batch or simple span processing, and optionally, a custom span exporter.

```rust
use trasy_error::TelemetryConfig;

let config = TelemetryConfig {
    service_name: "my-awesome-service".to_string(),
    endpoint: "http://my-telemetry-collector:4318".to_string(),
    use_batch: true,
    oltp_exporter: None, // Use default OTLP exporter
};
```

#### 2. Set Up OpenTelemetry

Pass the configuration to the `setup_opentelemetry` function to initialize the telemetry. This function sets up the tracing layer that you can then use with the `tracing` subscriber.

```rust
use trasy_error::setup_opentelemetry;

let telemetry_layer = setup_opentelemetry(config).await.expect("Failed to set up OpenTelemetry");

// Now you can use `telemetry_layer` with your tracing subscriber setup
```

### Example: Full Setup with Tracing Subscriber

Here is a complete example that shows how to set up tracing using `trasy_error` with OpenTelemetry and `tracing_subscriber`.

```rust
use trasy_error::{TelemetryConfig, setup_opentelemetry};
use tracing_subscriber::{layer::SubscriberExt, Registry};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = TelemetryConfig::default().with_oltp_exporter(
        opentelemetry_otlp::new_exporter().http().with_endpoint("http://localhost:4318")
    );

    let telemetry_layer = setup_opentelemetry(config).await?;

    let subscriber = Registry::default()
        .with(telemetry_layer)
        .with(tracing_subscriber::fmt::layer());

    tracing::subscriber::set_global_default(subscriber)?;

    // Your application code here
    tracing::info!("Application started");

    Ok(())
}
```

### Custom Exporters

If you need to use a custom exporter, configure it as part of your `TelemetryConfig`:

```rust
let custom_exporter = opentelemetry_otlp::new_exporter()
    .grpc()
    .with_endpoint("my-custom-endpoint:4317");

let config = TelemetryConfig {
    service_name: "my-service".to_string(),
    use_batch: false,
    oltp_exporter: Some(custom_exporter.into()),
    ..Default::default()
};
```

### Note

- Make sure your OpenTelemetry collector or backend is properly configured to receive telemetry data from your application.
- Adjust the OpenTelemetry setup according to your specific environment needs and the OpenTelemetry SDK documentation.

This integration allows `TrasyError` to be a powerful tool in not only handling errors but also in observing and diagnosing them in distributed systems.

Certainly! To incorporate the usage of your Docker setup with Jaeger into the `README.md`, you can provide a detailed guide on how to run the Jaeger instance using Docker and how to connect it with your application. Below is the section you can add to your `README.md` to cover this:

---

## Integrating Jaeger for Tracing Visualization

`Trasy` is configured to work seamlessly with Jaeger, a distributed tracing system. By using the provided Docker configuration, you can easily set up a Jaeger instance to visualize traces collected from your application. Here's how to get started:

### Setting Up Jaeger with Docker

To start the Jaeger container which will collect and visualize your application's tracing data, follow these steps:

1. Ensure Docker and Docker Compose are installed on your system. For installation instructions, see [Docker](https://docs.docker.com/get-docker/) and [Docker Compose](https://docs.docker.com/compose/install/).
2. Create a `docker-compose.yml` file with the following content:

    ```yaml
    version: '3.8'

    services:
      jaeger:
        image: jaegertracing/all-in-one:1.56
        container_name: jaeger
        ports:
          - "6831:6831/udp"   # Jaeger Thrift Compact Protocol
          - "6832:6832/udp"   # Jaeger Thrift Binary Protocol
          - "16686:16686"     # Jaeger UI
          - "14268:14268"     # Jaeger HTTP collector
          - "4317:4317"       # OTLP gRPC port
          - "4318:4318"       # OTLP gRPC http port
        environment:
          - COLLECTOR_OTLP_ENABLED=true
    ```

3. Run the following command in the directory where your `docker-compose.yml` is located:

    ```bash
    docker-compose up -d
    ```

   This command will download the Jaeger image and start the Jaeger service.

### Connecting Your Application to Jaeger

To send traces from your application to Jaeger, configure the `TelemetryConfig` to use the correct endpoint. Here’s an example using the default setup provided in the Docker configuration:

```rust
let config = TelemetryConfig {
    service_name: "my-awesome-service".to_string(),
    endpoint: "http://localhost:4318",
    use_batch: true,
    oltp_exporter: None, // This will use the default OTLP HTTP exporter
};

let telemetry_layer = setup_opentelemetry(config).await.expect("Failed to set up OpenTelemetry");
```

### Viewing Traces in Jaeger

After running your application configured to send traces to Jaeger, you can view these traces by:

1. Opening a web browser.
2. Navigating to `http://localhost:16686`.

This URL is the Jaeger UI, where you can query and visualize traces collected from your application.

### Note

- Make sure the port numbers in the Jaeger Docker setup match those expected by your application’s telemetry configuration.
- If running within different Docker networks or on different machines, ensure network connectivity between your application and the Jaeger service.

This setup provides a powerful way to visualize and debug the behavior of your distributed applications.

## Contributing

Contributions to `Trasy` are welcome! Here are some ways you can contribute:

- Reporting bugs
- Suggesting enhancements
- Adding more integrations and features
- Improving documentation

Please feel free to fork the repository and submit pull requests.
