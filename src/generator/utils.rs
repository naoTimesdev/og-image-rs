use crate::env::get_env;

pub fn get_generator_host() -> String {
    get_env("SERVER_HOSTNAME").unwrap_or_else(|_| {
        tracing::warn!("SERVER_HOSTNAME is not set. Using HOST:PORT instead.");

        let host_at = get_env("HOST").unwrap_or("127.0.0.1".to_string());
        let port_at = get_env("PORT").unwrap_or("12460".to_string());

        format!("{}:{}", host_at, port_at)
    })
}
