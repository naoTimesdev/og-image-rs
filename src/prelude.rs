use std::net::IpAddr;

use axum::{
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
};
use reqwest::header::GetAll;

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
        let x_forwarded_for: Vec<IpAddr> = parse_specific_headers(&val.get_all("x-forwarded-for"));
        let forwarded: Vec<IpAddr> = parse_specific_headers(&val.get_all(header::FORWARDED));
        let x_real_ip: Vec<IpAddr> = parse_specific_headers(&val.get_all("x-real-ip"));
        let cf_connecting_ip: Vec<IpAddr> =
            parse_specific_headers(&val.get_all("cf-connecting-ip"));
        let cf_connecting_ipv6: Vec<IpAddr> =
            parse_specific_headers(&val.get_all("cf-connecting-ipv6"));

        let mut ip_address: Vec<IpAddr> = vec![];
        ip_address.extend(cf_connecting_ip);
        ip_address.extend(cf_connecting_ipv6);
        ip_address.extend(x_forwarded_for);
        ip_address.extend(forwarded);
        ip_address.extend(x_real_ip);

        PlausibleMetadata {
            user_agent,
            ip_address,
        }
    }
}

fn parse_specific_headers(headers: &GetAll<HeaderValue>) -> Vec<IpAddr> {
    headers
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
        .collect()
}

pub fn is_private_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ipv4) => {
            if ipv4.is_private()
                || ipv4.is_loopback()
                || ipv4.is_link_local()
                || ipv4.is_unspecified()
                || ipv4.is_broadcast()
                || ipv4.is_documentation()
                || ipv4.is_multicast()
            {
                true
            } else {
                false
            }
        }
        IpAddr::V6(ipv6) => {
            if ipv6.is_loopback() || ipv6.is_multicast() || ipv6.is_unspecified() {
                true
            } else {
                false
            }
        }
    }
}
