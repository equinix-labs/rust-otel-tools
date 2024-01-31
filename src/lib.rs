use opentelemetry::trace::Tracer;

/// The expected environtment variable for carrying around W3C traceparents
/// among shell scripts
pub static TRACEPARENT: &str = "TRACEPARENT";

/// Attempt to parse a valid traceparent from the [`TRACEPARENT`] environment
/// variable
pub fn read_traceparent() -> Result<traceparent::Traceparent, anyhow::Error> {
    match std::env::var(TRACEPARENT) {
        Ok(val) => {
            let tp = traceparent::parse(&val);
            match tp {
                Ok(t) => Ok(t),
                Err(e) => Err(e),
            }
        }
        Err(e) => Err(e.into()),
    }
}

/// If the provided traceparent is valid and different from our current [`TRACEPARENT`]
/// variable, update the variable. Return whether or not we made a change.
pub fn update_traceparent(new_traceparent: String) -> bool {
    let current = match read_traceparent() {
        Ok(c) => c,
        Err(_) => traceparent::make(false), // make a bogus one for comparison
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
        Ok(tp) => {
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
        Err(_) => tracer.start(name),
    };
    opentelemetry::trace::mark_span_as_active(span)
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
