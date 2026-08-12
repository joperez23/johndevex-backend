use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub server_host: String,
    pub server_port: u16,
    pub database_url: String,
    pub request_timeout_secs: u64,
}

impl Config {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();

        let server_host = env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        let server_port = env::var("SERVER_PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse::<u16>()
            .expect("SERVER_PORT must be a valid u16 port number");

        let database_url =
            env::var("DATABASE_URL").expect("DATABASE_URL environment variable must be set");

        let request_timeout_secs = env::var("REQUEST_TIMEOUT_SECS")
            .unwrap_or_else(|_| "15".to_string())
            .parse::<u64>()
            .unwrap_or(15);

        Self {
            server_host,
            server_port,
            database_url,
            request_timeout_secs,
        }
    }
}
