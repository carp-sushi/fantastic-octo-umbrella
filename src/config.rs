use percent_encoding::NON_ALPHANUMERIC;
use sqlx::postgres::PgPoolOptions;
use sqlx::Executor;
use std::env;
use std::net::SocketAddr;
use std::sync::Arc;

/// Configuration settings
#[derive(Debug)]
pub struct Config {
    pub grpc_listen_addr: SocketAddr,
    pub db_max_connections: u32,
    pub db_host: String,
    pub db_port: u16,
    pub db_user: String,
    pub db_password: String,
    pub db_database: String,
    pub db_schema: String,
}

impl Config {
    /// Create a new config.
    pub fn new() -> Self {
        // gRPC server settings
        let listen_addr = env::var("GRPC_LISTEN_ADDR").unwrap_or("0.0.0.0:9090".into());
        let grpc_listen_addr = listen_addr.parse().expect("Unable to parse listen_addr");

        // database settings
        let db_max_connections = env::var("DB_MAX_CONNECTIONS")
            .unwrap_or("10".to_owned())
            .parse()
            .expect("DB_MAX_CONNECTIONS could not be parsed");
        let db_host = env::var("DB_HOST").expect("DB_HOST not set");
        let db_port = env::var("DB_PORT")
            .unwrap_or("5432".to_owned())
            .parse()
            .expect("DB_PORT could not be parsed");
        let db_user = env::var("DB_USER").expect("DB_USER not set");
        let db_password = env::var("DB_PASS").expect("DB_PASS not set");
        let db_database = env::var("DB_NAME").expect("DB_NAME not set");
        let db_schema = env::var("DB_SCHEMA").expect("DB_SCHEMA not set");

        // Config
        Self {
            grpc_listen_addr,
            db_max_connections,
            db_host,
            db_port,
            db_user,
            db_password,
            db_database,
            db_schema,
        }
    }

    pub fn db_connection_string(&self) -> String {
        let bytes = self.db_password.as_bytes();
        let password = percent_encoding::percent_encode(bytes, NON_ALPHANUMERIC);
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.db_user, password, self.db_host, self.db_port, self.db_database,
        )
    }

    pub fn db_pool_opts(&self) -> PgPoolOptions {
        let schema = Arc::new(self.db_schema.clone());
        PgPoolOptions::new()
            .max_connections(self.db_max_connections)
            .after_connect(move |conn, _meta| {
                let schema = Arc::clone(&schema);
                Box::pin(async move {
                    conn.execute(format!("SET search_path = '{}';", schema).as_ref())
                        .await?;
                    Ok(())
                })
            })
    }
}
