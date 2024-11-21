<!-- cargo-rdme start -->

# rust-otel-tools

Some reusable otel code for Equinix rust applications

Example:

```rust
use opentelemetry::trace::Tracer;

#[tracing::instrument(err)]
async fn something(message: String) -> Result<(), Box<dyn std::error::Error>> {
    // This will mark the span as an error even though it returns Ok(())
    tracing::error!("Error: {}", message);
    Ok(())
}

#[tokio::main]
async fn main() {
    // Set up the otel exporter based on the the otlp exporter environment variables
    // <https://opentelemetry.io/docs/languages/sdk-configuration/otlp-exporter/>
    let _ = equinix_otel_tools::init("example-service");

    // Set up a new active span, parsing the TRACEPARENT environment variable
    // if it's valid
    let guard = equinix_otel_tools::start_with_traceparent("example");

    // call an instrumented function
    something("Hello World".to_string()).await;

    drop(guard);
    opentelemetry::global::shutdown_tracer_provider();
}
```

<!-- cargo-rdme end -->
