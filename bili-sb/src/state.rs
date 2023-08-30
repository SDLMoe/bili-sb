use std::{sync::Arc, time::Duration};

use anyhow::Context;
use axum::extract::State;
use diesel_async::{pooled_connection::AsyncDieselConnectionManager, AsyncPgConnection};
use http::Uri;
use log::info;
use tokio::sync::OnceCell;

use crate::{
  client::{self, *},
  data::RespCode,
  error::*,
};

pub type AppState = State<Arc<App>>;

#[derive(Clone, Debug)]
pub struct App {
  bili_channel: OnceCell<tonic::transport::Channel>,
  db_pool: PgAsyncPool,
}

impl App {
  pub async fn new(database_url: &str) -> anyhow::Result<Self> {
    Uri::try_from(database_url.clone()).context(
      "Invalid uri for database url, example: postgres://user:paSsw0rD@localhost:3213/bilisb",
    )?;
    log::info!("Connecting to database `{}`", database_url);
    let db_config = PgConnectionManager::new(database_url);
    let pool: PgAsyncPool = bb8::Pool::builder()
      .connection_timeout(Duration::from_secs(3))
      .build(db_config)
      .await
      .with_context(|| format!("Failed to connect database, url: `{}`", database_url))?;

    Ok(Self {
      bili_channel: Default::default(),
      db_pool: pool,
    })
  }

  #[allow(dead_code)]
  pub async fn bili(&self) -> AppResult<tonic::transport::Channel> {
    self
      .bili_channel
      .get_or_try_init(|| async {
        info!(
          "Connecting bilibili grpc server `{}`",
          BILI_GRPC_FAILOVER_URL
        );
        client::connect(BILI_GRPC_FAILOVER_URL).await
      })
      .await
      .cloned()
      .with_app_error(RespCode::BILI_CLIENT_ERROR)
      .into_app_result()
  }

  pub async fn db_con(&self) -> AppResult<PooledPgCon<'_>> {
    self
      .db_pool
      .get()
      .await
      .context("Failed to get pooled database connection")
      .with_app_error(RespCode::DATABASE_ERROR)
      .into_app_result()
  }
}

pub type PgAsyncPool = bb8::Pool<PgConnectionManager>;
pub type PgConnectionManager = AsyncDieselConnectionManager<AsyncPgConnection>;
pub type PooledPgCon<'a> = bb8::PooledConnection<'a, PgConnectionManager>;
pub type Bb8DieselRunError = bb8::RunError<diesel_async::pooled_connection::PoolError>;
#[allow(dead_code)]
pub type PgGetAsyncConResult<'a> = Result<PooledPgCon<'a>, Bb8DieselRunError>;
