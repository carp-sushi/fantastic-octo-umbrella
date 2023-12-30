use sqlx::postgres::PgPool;
use sqlx::Error;
use std::sync::Arc;
use tokio::time::{self, Duration};
use tonic_health::{
    server::HealthReporter,
    ServingStatus::{NotServing, Serving},
};

pub struct Health {}

impl Health {
    pub fn new() -> Self {
        Self {}
    }
}

impl Health {
    /// Check that the repo can query the database
    pub(crate) async fn health_check_query(db: &PgPool) -> Result<(), Error> {
        log::debug!("Health::health_check_query");
        sqlx::query("SELECT 1").fetch_one(db).await?;
        Ok(())
    }

    /// Health check for the gRPC server. Makes sure the database is accessible.
    pub async fn check(mut reporter: HealthReporter, db: Arc<PgPool>) {
        log::info!("Starting health check");
        let db = db.as_ref();
        loop {
            time::sleep(Duration::from_secs(5)).await;
            match Health::health_check_query(db).await {
                Ok(_) => reporter.set_service_status("", Serving).await,
                Err(err) => {
                    log::error!("Health check failed: {}", err.to_string());
                    reporter.set_service_status("", NotServing).await;
                }
            }
        }
    }
}
