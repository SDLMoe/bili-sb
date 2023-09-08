use anyhow::Context;
use axum_client_ip::SecureClientIpSource;
use governor::{clock::QuantaInstant, middleware::NoOpMiddleware};
use http::Method;
use std::{
  fs::File,
  io::{BufReader, Read},
  num::{NonZeroU32, NonZeroU64, NonZeroUsize},
  path::Path,
  time::Duration,
};
use tower_governor::governor::{GovernorConfig, GovernorConfigBuilder};

use serde::Deserialize;

use crate::layer::SecureIpExtractor;

mod default;

use default::*;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
  #[serde(default = "ip_source_default")]
  pub ip_source: SecureClientIpSource,
  #[serde(default)]
  pub ratelimit: Ratelimits,
  #[serde(default)]
  pub pow: PowConfig,
}

impl Config {
  pub fn load<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
    let path = path.as_ref();
    let file = File::open(path)
      .with_context(|| format!("Failed to open config file `{}`", path.to_string_lossy()))?;
    let size_hint = file
      .metadata()
      .map(|metadata| metadata.len() as usize)
      .unwrap_or(8 * 1024);
    let mut buf = String::with_capacity(size_hint);
    BufReader::new(file)
      .read_to_string(&mut buf)
      .with_context(|| format!("Failed to read config file `{}`", path.to_string_lossy()))?;
    toml::from_str(&buf).with_context(|| {
      format!(
        "Failed to deserilaize config file as file, {}",
        path.to_string_lossy()
      )
    })
  }

  pub fn ratelimit_get_conf(
    &self,
  ) -> GovernorConfig<SecureIpExtractor, NoOpMiddleware<QuantaInstant>> {
    GovernorConfigBuilder::default()
      .key_extractor(SecureIpExtractor)
      .methods(vec![Method::GET])
      .period(self.ratelimit.get.period.into())
      .burst_size(self.ratelimit.get.burst_size.get())
      .finish()
      .unwrap() // unlikely to panic, we already use NonZero* types
  }

  pub fn ratelimit_post_conf(
    &self,
  ) -> GovernorConfig<SecureIpExtractor, NoOpMiddleware<QuantaInstant>> {
    GovernorConfigBuilder::default()
      .key_extractor(SecureIpExtractor)
      .methods(vec![Method::POST])
      .period(self.ratelimit.post.period.into())
      .burst_size(self.ratelimit.post.burst_size.get())
      .finish()
      .unwrap() // unlikely to panic, we already use NonZero* types
  }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Ratelimits {
  #[serde(default = "ratelimit_get_default")]
  pub get: RatelimitConfig,
  #[serde(default = "ratelimit_post_default")]
  pub post: RatelimitConfig,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct RatelimitConfig {
  #[serde(flatten)]
  pub period: RatelimitPeriod,
  #[serde(alias = "burst")]
  pub burst_size: NonZeroU32,
}

#[derive(Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "kebab-case")]
#[allow(clippy::enum_variant_names)]
pub enum RatelimitPeriod {
  PerSecond(NonZeroU64),
  PerMs(NonZeroU64),
  PerNano(NonZeroU64),
}

impl From<RatelimitPeriod> for Duration {
  fn from(val: RatelimitPeriod) -> Self {
    use RatelimitPeriod as P;
    match val {
      P::PerSecond(i) => Duration::from_secs(i.get()),
      P::PerMs(i) => Duration::from_millis(i.get()),
      P::PerNano(i) => Duration::from_nanos(i.get()),
    }
  }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct PowConfig {
  #[serde(default = "pow_enabled_default")]
  pub enabled: bool,
  #[serde(alias = "salt-len")]
  #[serde(default = "pow_salt_size_default")]
  pub salt_size: NonZeroUsize,
  #[serde(default = "pow_cost_default")]
  pub cost: u32,
  #[serde(alias = "ts-delta")]
  #[serde(default = "pow_timestamp_delta_default")]
  pub timestamp_delta: u64,
}
