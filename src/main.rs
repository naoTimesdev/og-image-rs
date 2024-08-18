use std::{net::IpAddr, str::FromStr, sync::Arc};

use crate::env::get_env;
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use reqwest::header::HeaderMap;
use tokio::{net::TcpListener, sync::Mutex, task::JoinHandle};
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing::debug;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod env;
mod generator;
mod prelude;
mod routes;

#[derive(Clone)]
pub struct AppState {
    join_handle: Arc<Mutex<Option<JoinHandle<()>>>>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct PlausibleEvent {
    pub name: String,
    pub url: String,
    pub props: Option<serde_json::Value>,
    pub domain: Option<String>,
}

pub struct PlausibleMetadata {
    pub user_agent: String,
    pub ip_address: Option<IpAddr>,
}

impl Default for PlausibleEvent {
    fn default() -> Self {
        Self {
            name: "pageview".to_string(),
            url: "".to_owned(),
            props: None,
            domain: None,
        }
    }
}

pub async fn report_plausible_event(
    state: AppState,
    mut event: PlausibleEvent,
    metadata: PlausibleMetadata,
) {
    let plausible_endpoint = get_env("PLAUSIBLE_ENDPOINT").unwrap_or("".to_string());
    if plausible_endpoint.is_empty() {
        return;
    }
    let plausible_domain = get_env("PLAUSIBLE_DOMAIN").unwrap_or("".to_string());
    if plausible_domain.is_empty() {
        return;
    }
    let mut lock = state.join_handle.lock().await;
    *lock = Some(tokio::spawn(async move {
        debug!("Reporting plausible event: {:?}", event);
        event.domain = Some(plausible_domain);

        let client = reqwest::ClientBuilder::new()
            .user_agent(metadata.user_agent)
            .build()
            .unwrap();
        let mut headers = HeaderMap::new();

        let real_ip = metadata
            .ip_address
            .clone()
            .unwrap_or(IpAddr::from_str("127.0.0.1").unwrap());

        headers.insert("X-Forwarded-For", real_ip.to_string().parse().unwrap());
        headers.insert("Content-Type", "application/json".parse().unwrap());

        let body = serde_json::to_string(&event).unwrap();

        debug!("Sending plausible event: {} // {:?}", body, headers);
        // post
        let _ = client
            .post(format!("{}/api/event", plausible_endpoint))
            .body(body)
            .headers(headers)
            .send()
            .await
            .unwrap();
        debug!("Sent plausible event: {:?}", event);
    }))
}

#[tokio::main]
async fn main() {
    let state = AppState {
        join_handle: Arc::new(Mutex::new(None)),
    };
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
        .route("/_/health", get(|| async { "ok" }))
        .route(
            "/music/bandcamp",
            get(routes::music_thumb::handle_bandcamp_thumb),
        )
        .route(
            "/music/soundcloud/:artist/:title",
            get(routes::music_thumb::handle_soundcloud_thumb),
        )
        .route(
            "/music/ytm/:id",
            get(routes::music_thumb::handle_youtube_music_thumb),
        )
        .route(
            "/_/template/user_card",
            get(routes::template::handle_template_user_card),
        )
        .route(
            "/generator/user",
            get(routes::generator::handle_generator_user_card),
        )
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::new().allow_origin(Any))
        .with_state(state);
    let app = app.fallback(handle_404);

    let host_at = get_env("HOST").unwrap_or("127.0.0.1".to_string());
    let port_at = get_env("PORT").unwrap_or("12460".to_string());

    // run it
    let listener = TcpListener::bind(format!("{}:{}", host_at, port_at))
        .await
        .unwrap();
    tracing::info!(
        "🚀 Fast serving at: http://{}",
        listener.local_addr().unwrap()
    );
    axum::serve(listener, app).await.unwrap();
}

async fn index() -> &'static str {
    "</Mutex> Made for naoTimes by @noaione</>"
}

async fn handle_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, Html("<h2>404 Not Found</h2>"))
}
