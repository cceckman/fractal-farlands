use axum::http::Request;
use ff_web::root_routes;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn main() {
    let web_rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("could not construct Tokio runtime");

    // Tracing config, from the Axum example:
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                // axum logs rejections from built-in extractors with the `axum::rejection`
                // target, at `TRACE` level. `axum::rejection=trace` enables showing those events
                "debug,axum::rejection=trace".into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let trace = TraceLayer::new_for_http().make_span_with(|request: &Request<_>| {
        // Log the path and query string
        let query = request
            .uri()
            .path_and_query()
            .map(|v| v.as_str())
            .unwrap_or("");
        tracing::info_span!(
            "http_request",
            method = ?request.method(),
            query,
        )
    });

    let server = async move {
        let app = root_routes()
            .expect("failed to start rendering pool")
            .layer(trace);
        const ADDR: &str = "0.0.0.0:3000";
        let listener = tokio::net::TcpListener::bind(ADDR).await?;
        tracing::info!("listening at {:?}", ADDR);
        axum::serve(listener, app).await
    };
    web_rt.block_on(server).expect("server terminated");
}
