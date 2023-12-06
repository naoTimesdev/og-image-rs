use axum::{
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use tokio::net::TcpListener;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod prelude;
mod routes;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                "naotimes_open_graph=debug,tower_http=debug,axum::rejection=trace".into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let app = Router::new()
        .route("/", get(index))
        .route("/large", get(routes::naotimes_og::handle_og_image_request))
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::new().allow_origin(Any));
    let app = app.fallback(handle_404);

    // run it
    let listener = TcpListener::bind("127.0.0.1:12460").await.unwrap();
    tracing::info!(
        "🚀 Fast serving at: http://{}",
        listener.local_addr().unwrap()
    );
    axum::serve(listener, app).await.unwrap();
}

async fn index() -> &'static str {
    "</> Made for naoTimes by @noaione</>"
}

async fn handle_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, Html("<h2>404 Not Found</h2>"))
}
