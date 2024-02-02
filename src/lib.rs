use opentelemetry::trace::Tracer;

/// The expected environtment variable for carrying around W3C traceparents
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

/// Start up a new otel span using name as the tracer name.
/// If a valid [`TRACEPARENT`] environment variable is found it will be used
/// to assemble the parent context for the span to propagate the trace
/// information.
pub fn start_with_traceparent(name: &'static str) -> opentelemetry::ContextGuard {
    let tracer = opentelemetry::global::tracer(name);
    let span = match read_traceparent() {
        Some(tp) => {
            let parent_spancontext = opentelemetry::trace::SpanContext::new(
                tp.trace_id().into(),
                tp.parent_id().into(),
                opentelemetry::trace::TraceFlags::SAMPLED, // tp.flags().into(),
                false,                                     // XXX????
                opentelemetry::trace::TraceState::NONE,
            );
            let trace_context = opentelemetry::Context::new();
            let parent_context = opentelemetry::trace::TraceContextExt::with_remote_span_context(
                &trace_context,
                parent_spancontext,
            );
            tracer.start_with_context(name, &parent_context)
        }
        None => tracer.start(name),
    };
    opentelemetry::trace::mark_span_as_active(span)
}

/// Generate a traceparent string for propagation
/// Reference: [https://github.com/open-telemetry/opentelemetry-rust/blob/0101233973ca8d635970bf7231c7eccda0e9764e/opentelemetry-sdk/src/propagation/trace_context.rs#L116-L123]
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
/// XXX currently hardcoding the OTLP endpoint which is wrong.
pub fn init(name: &'static str) -> Result<(), Box<dyn std::error::Error>> {
    // XXX setting some variables manually. remove later.
    std::env::set_var("OTEL_SERVICE_NAME", name);
    std::env::set_var("OTEL_EXPORTER_OTLP_ENDPOINT", "http://localhost:4317");
    init_tracing_opentelemetry::tracing_subscriber_ext::init_subscribers()
        .expect("init subscribers");
    Ok(())
}
