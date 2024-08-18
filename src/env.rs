use dotenvy::dotenv;

pub fn get_env(key: &str) -> Result<String, std::env::VarError> {
    dotenv().ok();

    std::env::var(key)
}
