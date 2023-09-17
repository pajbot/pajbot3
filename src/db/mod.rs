pub mod models;

use crate::Config;
use deadpool_postgres::PoolConfig;
use deadpool_postgres::RecyclingMethod;
use deadpool_postgres::{ManagerConfig, Object, PoolError};
use rustls::OwnedTrustAnchor;
use rustls::RootCertStore;
use std::ops::DerefMut;
use tokio_postgres_rustls::MakeRustlsConnect;

#[derive(Clone)]
pub struct DataStorage {
    db: deadpool_postgres::Pool,
}

pub async fn connect_to_postgresql(config: &Config) -> DataStorage {
    let pg_config = tokio_postgres::Config::from(config.database.clone());
    tracing::debug!("PostgreSQL config: {:#?}", pg_config);

    let mgr_config = ManagerConfig {
        recycling_method: RecyclingMethod::Fast,
    };
    let pool_config = PoolConfig {
        max_size: config.database.pool.max_size,
        timeouts: deadpool_postgres::Timeouts::from(config.database.pool),
    };

    let mut root_certificates = RootCertStore::empty();
    let trust_anchors = webpki_roots::TLS_SERVER_ROOTS.iter().map(|trust_anchor| {
        OwnedTrustAnchor::from_subject_spki_name_constraints(
            trust_anchor.subject,
            trust_anchor.spki,
            trust_anchor.name_constraints,
        )
    });
    root_certificates.add_server_trust_anchors(trust_anchors);

    let tls_config = rustls::ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(root_certificates) // TODO support custom root certificates as well
        .with_no_client_auth(); // TODO support client auth if needed

    let tls = MakeRustlsConnect::new(tls_config);

    let manager = deadpool_postgres::Manager::from_config(pg_config, tls, mgr_config);
    DataStorage {
        db: deadpool_postgres::Pool::builder(manager)
            .config(pool_config)
            .runtime(deadpool_postgres::Runtime::Tokio1)
            .build()
            .unwrap(),
    }
}

mod migrations {
    use refinery::embed_migrations;
    // refers to the "migrations" directory in the project root
    embed_migrations!("migrations");
}

impl DataStorage {
    pub async fn run_migrations(&self) -> Result<(), Box<dyn std::error::Error>> {
        migrations::migrations::runner()
            .run_async(self.db.get().await?.as_mut().deref_mut())
            .await?;
        Ok(())
    }

    pub async fn get(&self) -> Result<Object, PoolError> {
        self.db.get().await
    }
}
