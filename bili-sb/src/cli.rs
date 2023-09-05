use clap::{
  builder::{styling::AnsiColor, Styles},
  Parser, ValueHint,
};
use std::path::PathBuf;

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
pub struct Args {
  // `[::]` binds to IPv6 and IPv4 at the same time
  // See: https://github.com/tokio-rs/axum/discussions/834
  /// Address to bind
  #[arg(short = 'i', long = "addr")]
  #[arg(default_value = "[::]:8402")]
  #[arg(env = "BILI_SB_ADDR")]
  pub addr: String,
  /// Database url to connect
  #[arg(short = 'd', long = "database-url")]
  #[arg(env = "BILI_SB_DATABASE_URL")]
  pub database_url: String,
  /// Sets a custom config file
  #[arg(short = 'c', long = "config", value_name = "FILE")]
  #[arg(value_hint = ValueHint::FilePath)]
  #[arg(env = "BILI_SB_CONFIG")]
  pub config: Option<PathBuf>,
}
