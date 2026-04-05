use actix_web::dev::{ServiceRequest, ServiceResponse};
use tracing_actix_web::{DefaultRootSpanBuilder, RootSpanBuilder};

pub struct RequestIdSpanBuilder;

impl RootSpanBuilder for RequestIdSpanBuilder {
    fn on_request_start(request: &ServiceRequest) -> tracing::Span {
        let span = DefaultRootSpanBuilder::on_request_start(request);
        if let Some(id) = request
            .headers()
            .get("X-Request-ID")
            .and_then(|v| v.to_str().ok())
        {
            span.record("request_id", id);
        }
        span
    }

    fn on_request_end<B: actix_web::body::MessageBody>(
        span: tracing::Span,
        outcome: &Result<ServiceResponse<B>, actix_web::Error>,
    ) {
        DefaultRootSpanBuilder::on_request_end(span, outcome);
    }
}
