use axum::{
    http::{header, HeaderMap, StatusCode},
    response::IntoResponse,
    response::Response,
};

use crate::PlausibleMetadata;

struct AppError(anyhow::Error);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.0),
        )
            .into_response()
    }
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

impl Into<PlausibleMetadata> for HeaderMap {
    fn into(self) -> PlausibleMetadata {
        let user_agent = self
            .get(header::USER_AGENT)
            .map(|v| v.to_str().unwrap_or_default().to_string())
            .unwrap_or_default();
        let ip_address = self
            .get("x-forwarded-for")
            .map(|v| v.to_str().unwrap_or_default().to_string())
            .unwrap_or_default();

        PlausibleMetadata {
            user_agent,
            ip_address,
        }
    }
}
