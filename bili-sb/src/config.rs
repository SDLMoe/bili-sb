use anyhow::Context;
use axum_client_ip::SecureClientIpSource;
use governor::{clock::QuantaInstant, middleware::NoOpMiddleware};
use http::Method;
use std::{
  fs::File,
  io::{BufReader, Read},
  num::{NonZeroU32, NonZeroU64},
  path::Path,
  time::Duration,
};
use tower_governor::governor::{GovernorConfig, GovernorConfigBuilder};

use serde::Deserialize;

use crate::layer::SecureIpExtractor;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
  #[serde(default = "ip_source_default")]
  pub ip_source: SecureClientIpSource,
  #[serde(default)]
  pub ratelimit: Ratelimits,
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
}

impl Default for Config {
  fn default() -> Self {
    Self {
      ip_source: ip_source_default(),
      ratelimit: Default::default(),
    }
  }
}

fn ip_source_default() -> SecureClientIpSource {
  SecureClientIpSource::ConnectInfo
}

impl Config {
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

impl Default for Ratelimits {
  fn default() -> Self {
    Self {
      get: ratelimit_get_default(),
      post: ratelimit_post_default(),
    }
  }
}

fn ratelimit_get_default() -> RatelimitConfig {
  RatelimitConfig {
    period: RatelimitPeriod::PerMs(unsafe { NonZeroU64::new_unchecked(500) }),
    burst_size: unsafe { NonZeroU32::new_unchecked(50) },
  }
}

fn ratelimit_post_default() -> RatelimitConfig {
  RatelimitConfig {
    period: RatelimitPeriod::PerSecond(unsafe { NonZeroU64::new_unchecked(1) }),
    burst_size: unsafe { NonZeroU32::new_unchecked(20) },
  }
}

#[derive(Deserialize, Debug, Clone)]
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
