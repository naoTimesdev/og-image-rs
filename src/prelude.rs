use std::net::IpAddr;

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

impl From<HeaderMap> for PlausibleMetadata {
    fn from(val: HeaderMap) -> Self {
        let user_agent = val
            .get(header::USER_AGENT)
            .map(|v| v.to_str().unwrap_or_default().to_string())
            .unwrap_or_default();

        // Get rightmost IP address from X-Forwarded-For header
        let ip_addresses: Vec<IpAddr> = val
            .get_all("X-Forwarded-For")
            .iter()
            .filter_map(|v| {
                // parse into IpAddr
                match v.to_str() {
                    Ok(v) => match v.parse() {
                        Ok(v) => Some(v),
                        Err(_) => None,
                    },
                    Err(_) => None,
                }
            })
            .collect();

        let ip_address = ip_addresses.last().cloned();

        PlausibleMetadata {
            user_agent,
            ip_address,
        }
    }
}
