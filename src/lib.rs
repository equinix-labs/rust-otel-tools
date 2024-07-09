//! # rust-otel-tools
//!
//! Some reusable otel code for Equinix rust applications
//!
//! Example:
//!
//! ```rust
//! use opentelemetry::trace::Tracer;
//!
//! #[tracing::instrument(err)]
//! async fn something(message: String) -> Result<(), Box<dyn std::error::Error>> {
//!     // This will mark the span as an error even though it returns Ok(())
//!     tracing::error!("Error: {}", message);
//!     Ok(())
//! }
//!
//! #[tokio::main]
//! async fn main() {
//!     // Set up the otel exporter based on the the otlp exporter environment variables
//!     // <https://opentelemetry.io/docs/languages/sdk-configuration/otlp-exporter/>
//!     let _ = equinix_otel_tools::init("example-service");
//!
//!     // Set up a new active span, parsing the TRACEPARENT environment variable
//!     // if it's valid
//!     let guard = equinix_otel_tools::start_with_traceparent("example");
//!
//!     // call an instrumented function
//!     something("Hello World".to_string()).await;
//!
//!     drop(guard);
//!     opentelemetry::global::shutdown_tracer_provider();
//! }
//! ```

use opentelemetry::trace::Tracer;
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;

/// The expected environment variable for carrying around W3C traceparents
/// among shell scripts
pub static TRACEPARENT: &str = "TRACEPARENT";

/// Attempt to parse a valid traceparent from the [`TRACEPARENT`] environment
/// variable
pub fn read_traceparent() -> Option<traceparent::Traceparent> {
    match std::env::var(TRACEPARENT) {
        Ok(val) => {
            if let Ok(tp) = traceparent::parse(&val) {
                return Some(tp);
            }
            None
        }
        Err(_) => None,
    }
}

/// If the provided traceparent is valid and different from our current [`TRACEPARENT`]
/// variable, update the variable. Return whether or not we made a change.
pub fn update_traceparent(new_traceparent: String) -> bool {
    let current = match read_traceparent() {
        Some(c) => c,
        None => traceparent::make(false), // make a bogus one for comparison
    };
    if let Ok(tp) = traceparent::parse(&new_traceparent) {
        if tp == current {
            return false;
        }
        std::env::set_var("TRACEPARENT", new_traceparent);
        return true;
    }
    false
}

/// Start up a new otel span using name as the span name.
/// If a valid [`TRACEPARENT`] environment variable is found it will be used
/// to assemble the parent context for the span to propagate the trace
/// information.
pub fn start_with_traceparent(span_name: &'static str) -> opentelemetry::ContextGuard {
    // The use of empty string here will cause you to get a tracer named the same as what you
    // provided to our init function.
    let tracer = opentelemetry::global::tracer("");
    let span = match read_traceparent() {
        Some(tp) => {
            let parent_spancontext = opentelemetry::trace::SpanContext::new(
                tp.trace_id().into(),
                tp.parent_id().into(),
                opentelemetry::trace::TraceFlags::SAMPLED, // tp.flags().into(),
                false,                                     // TODO: should this be something else?
                opentelemetry::trace::TraceState::NONE,
            );
            let trace_context = opentelemetry::Context::new();
            let parent_context = opentelemetry::trace::TraceContextExt::with_remote_span_context(
                &trace_context,
                parent_spancontext,
            );
            tracer.start_with_context(span_name, &parent_context)
        }
        None => tracer.start(span_name),
    };
    opentelemetry::trace::mark_span_as_active(span)
}

/// [Generate a traceparent string for propagation](https://github.com/open-telemetry/opentelemetry-rust/blob/0101233973ca8d635970bf7231c7eccda0e9764e/opentelemetry-sdk/src/propagation/trace_context.rs#L116-L123)
pub fn generate_traceparent() -> Option<String> {
    // https://github.com/open-telemetry/opentelemetry-rust/blob/0101233973ca8d635970bf7231c7eccda0e9764e/opentelemetry-sdk/src/propagation/trace_context.rs#L28C1-L29C1
    const SUPPORTED_VERSION: u8 = 0;
    return opentelemetry::trace::get_active_span(|span| {
        let span_context = span.span_context();
        if span_context.is_valid() {
            return Some(format!(
                "{:02x}-{}-{}-{:02x}",
                SUPPORTED_VERSION,
                span_context.trace_id(),
                span_context.span_id(),
                span_context.trace_flags() & opentelemetry::trace::TraceFlags::SAMPLED
            ));
        }
        None
    });
}

/// A super-duper opinionated way to initialize otel tracing.
/// We will respect an existing OTEL_SERVICE_NAME environment variable,
/// but if it's absent, we set it based on what was passed in the call.
pub fn init(name: &'static str) -> Result<(), Box<dyn std::error::Error>> {
    match std::env::var("OTEL_SERVICE_NAME") {
        Ok(_) => (),
        Err(_) => std::env::set_var("OTEL_SERVICE_NAME", name),
    };

    if let Ok(()) = init_tracing_opentelemetry::tracing_subscriber_ext::init_subscribers() {
        return Ok(());
    } else {
        // Recreate the "temporary subscriber" setup from init-tracing-opentelemtry as a fallback
        let subscriber = tracing_subscriber::registry()
            .with(init_tracing_opentelemetry::tracing_subscriber_ext::build_loglevel_filter_layer())
            .with(init_tracing_opentelemetry::tracing_subscriber_ext::build_logger_text());
        tracing::subscriber::set_global_default(subscriber)?;
        tracing::warn!("Tracing setup failed. Falling back to local logging.");
    }
    Ok(())
}
