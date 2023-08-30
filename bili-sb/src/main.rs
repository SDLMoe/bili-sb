use std::{net::SocketAddr, sync::Arc};

use anyhow::Context;
use axum::{
  body::Body,
  response::{IntoResponse, Response},
  routing::{get, post},
  Router,
};
use axum_client_ip::{SecureClientIp, SecureClientIpSource};
use clap::{
  builder::{styling::AnsiColor, Styles},
  Parser,
};
use diesel_async::RunQueryDsl;
use http::{Method, Request, StatusCode};
use indoc::concatdoc;
use log::info;
use tower_http::compression::CompressionLayer;

use crate::{data::*, error::*, state::*};

#[allow(dead_code)]
mod client;
mod data;
mod db;
mod error;
mod macros;
mod state;

fn clap_v3_styles() -> Styles {
  Styles::styled()
    .header(AnsiColor::Yellow.on_default())
    .usage(AnsiColor::Green.on_default())
    .literal(AnsiColor::Green.on_default())
    .placeholder(AnsiColor::Green.on_default())
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(styles(clap_v3_styles()))]
struct Args {
  // `[::]` binds to IPv6 and IPv4 at the same time
  // See: https://github.com/tokio-rs/axum/discussions/834
  /// Address to bind
  #[arg(short = 'i', long = "addr")]
  #[arg(default_value = "[::]:8402")]
  #[arg(env = "BILI_SB_ADDR")]
  addr: String,
  #[arg(short = 'd', long = "database-url")]
  #[arg(env = "BILI_SB_DATABASE_URL")]
  database_url: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  const LOG_ENV: &str = "BILI_SB_LOG";
  if std::env::var_os(LOG_ENV).is_none() {
    #[cfg(not(feature = "dev"))]
    std::env::set_var(LOG_ENV, "info");
    #[cfg(feature = "dev")]
    std::env::set_var(LOG_ENV, "debug");
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

  let args = Args::parse();
  let addr = tokio::net::lookup_host(&args.addr)
    .await
    .with_context(|| format!("Cannot lookup DNS for addr: {}", args.addr))?
    .next()
    .with_context(|| format!("No DNS resp for addr: {}", args.addr))?;

  let router = Router::new()
    .route("/", get(root))
    .route("/user/create", post(user_create))
    .fallback(fallback)
    .with_state(Arc::new(App::new(&args.database_url).await?))
    .layer(CompressionLayer::new())
    // TODO: configurable
    .layer(SecureClientIpSource::ConnectInfo.into_extension());

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

async fn user_create(state: AppState, ip: SecureClientIp) -> AppResult<Resp<CreateUserData>> {
  let mut con = state.db_con().await?;
  let user = db::User::new(ip.0.into());
  let result = diesel::insert_into(db::users::table)
    .values(&user)
    .execute(&mut con)
    .await
    .context_into_app("Failed to insert")?;

  if result != 1 {
    return Err(app_err!(RespCode::DATABASE_ERROR, "Database insert failed"));
  }

  Ok(CreateUserData { uuid: user.id }.into())
}
