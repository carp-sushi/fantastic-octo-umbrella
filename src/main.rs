use todos::v1::{
    api::Todos, proto::todos_service_server::TodosServiceServer, repo::Repo, service::Service,
};
use todos::{config::Config, health::Health};

use sqlx::migrate::Migrator;
use std::error::Error;
use std::sync::Arc;
use tonic::transport::Server;

// Embed migrations into the gRPC server binary.
pub static MIGRATOR: Migrator = sqlx::migrate!();

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    // Load config
    let config = Config::new();
    log::debug!("Loaded config = {:?}", config);

    // Create pg connection pool
    let pool = config
        .db_pool_opts()
        .connect(config.db_connection_string().as_ref())
        .await?;

    log::info!("Running migrations");
    MIGRATOR.run(&pool).await?;

    // Arc up connection pool for async sharing across tasks
    let pool = Arc::new(pool);

    // Start health check task
    let (reporter, health_service) = tonic_health::server::health_reporter();
    tokio::spawn(Health::check(reporter, Arc::clone(&pool)));

    // Set up core logic for v1.
    let repo = Repo::new(Arc::clone(&pool));
    let service = Service::new(repo);
    let api = Todos::new(service);
    let todos_v1 = TodosServiceServer::new(api);

    // Serve gRPC API
    log::info!("Server listening on {}", config.grpc_listen_addr);
    Server::builder()
        .add_service(health_service)
        .add_service(todos_v1)
        .serve(config.grpc_listen_addr)
        .await?;

    Ok(())
}
