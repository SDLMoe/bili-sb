use std::{net::SocketAddr, sync::Arc, time::Duration};

use anyhow::Context;
use axum::{
  body::Body,
  response::{IntoResponse, Response},
  routing::{get, post},
  Router,
};
use axum_client_ip::SecureClientIp;
use clap::Parser;
use http::{Method, Request, StatusCode};
use indoc::concatdoc;
use log::info;
use rand::RngCore;
use tokio::time::sleep;
use tower_http::compression::CompressionLayer;
use uuid::Uuid;

use crate::config::Config;
#[allow(unused)]
use crate::{client::*, data::*, error::*, layer::*, routes::*, state::*};

mod cli;
#[allow(dead_code)]
mod client;
mod config;
mod data;
mod db;
mod error;
mod layer;
mod macros;
mod routes;
mod state;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  const LOG_ENV: &str = "BILI_SB_LOG";
  if std::env::var_os(LOG_ENV).is_none() {
    #[cfg(not(feature = "dev"))]
    std::env::set_var(LOG_ENV, "info");
    #[cfg(feature = "dev")]
    std::env::set_var(LOG_ENV, "warn");
  }

  pretty_env_logger::try_init_custom_env(LOG_ENV).context("Failed to init bili-sb logger")?;

  #[cfg(feature = "dev")]
  {
    log::info!("Running in development mode");
    if dotenvy::dotenv().is_err() {
      log::info!(
        ".env not found, you can create it at workspace root for better development experience."
      );
    }
  }

  let args = cli::Args::parse();

  let config = if let Some(path) = args.config {
    info!("Loading config {}", path.to_string_lossy());
    Config::load(path)?
  } else {
    Config::default()
  };

  let addr = tokio::net::lookup_host(&args.addr)
    .await
    .with_context(|| format!("Cannot lookup DNS for addr: {}", args.addr))?
    .next()
    .with_context(|| format!("No DNS resp for addr: {}", args.addr))?;

  let config = Arc::new(config);
  if config.pow.enabled {
    info!("PoW enabled: {:?}", &config.pow);
  } else {
    info!("PoW disabled!");
  }

  let state = Arc::new(App::new(&args.database_url, config).await?);
  let post_ratelimit_conf = Box::new(state.config.ratelimit_post_conf());
  info!(
    "[POST] ratelimit enabled: {:?}",
    &state.config.ratelimit.get
  );
  let get_ratelimit_conf = Box::new(state.config.ratelimit_get_conf());
  info!(
    "[GET ] ratelimit enabled: {:?}",
    &state.config.ratelimit.post
  );

  let router = Router::new()
    .route("/", get(root))
    .route("/pow/choose", post(pow_choose))
    .route("/user/create", post(user_create))
    .route("/segment/create", post(segment_create))
    .route("/segment/list", get(segment_list))
    .route("/segment/vote", post(segment_vote))
    .fallback(fallback)
    .with_state(Arc::clone(&state))
    .layer(CompressionLayer::new())
    .layer(axum::middleware::from_fn_with_state(
      Arc::clone(&state),
      pow_layer,
    ))
    .layer(ratelimit!(Box::leak(get_ratelimit_conf)))
    .layer(ratelimit!(Box::leak(post_ratelimit_conf)))
    .layer(state.config.ip_source.clone().into_extension());

  info!("Server is listening on {}", addr);
  axum::Server::try_bind(&addr)
    .context("Failed to bind address")?
    .serve(router.into_make_service_with_connect_info::<SocketAddr>())
    .await
    .context("Failed to launch server")?;

  Ok(())
}

async fn root() -> &'static str {
  concatdoc! {"
      Welcome! bili-sb api v", env!("CARGO_PKG_VERSION"), "

      Our homepage is at ", env!("CARGO_PKG_HOMEPAGE")
  }
}

async fn fallback(request: Request<Body>) -> Response {
  let uri = request.uri();
  let resp_code = StatusCode::NOT_FOUND;
  if request.method() == Method::GET {
    (resp_code, error_html(Some(uri), resp_code, "No such path")).into_response()
  } else {
    (resp_code, "Fatal: No router for such path").into_response()
  }
}
