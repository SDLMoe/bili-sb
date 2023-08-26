use anyhow::Context;
use axum::{routing::get, Router};
use clap::Parser;
use indoc::concatdoc;
use log::info;

pub mod client;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
  // `[::]` binds to IPv6 and IPv4 at the same time
  // See: https://github.com/tokio-rs/axum/discussions/834
  /// Address to bind
  #[arg(short = 'i', long = "addr")]
  #[arg(default_value = "[::]:8402")]
  addr: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let args = Args::parse();
  let addr = tokio::net::lookup_host(&args.addr)
    .await
    .with_context(|| format!("Cannot lookup DNS for addr: {}", args.addr))?
    .next()
    .with_context(|| format!("No DNS resp for addr: {}", args.addr))?;

  const LOG_ENV: &str = "BILI_SB_LOG";
  if std::env::var_os(LOG_ENV).is_none() {
    std::env::set_var(LOG_ENV, "info");
  }
  pretty_env_logger::try_init_custom_env(LOG_ENV).context("Failed to init bili-sb logger")?;

  let router = Router::new().route("/", get(root));

  info!("Server is listening on {}", addr);
  axum::Server::try_bind(&addr)
    .context("Faield to bind address")?
    .serve(router.into_make_service())
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
