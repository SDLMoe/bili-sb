use super::*;

impl Default for Config {
  fn default() -> Self {
    Self {
      ip_source: ip_source_default(),
      ratelimit: Default::default(),
      pow: Default::default(),
    }
  }
}

pub fn ip_source_default() -> SecureClientIpSource {
  SecureClientIpSource::ConnectInfo
}

impl Default for Ratelimits {
  fn default() -> Self {
    Self {
      get: ratelimit_get_default(),
      post: ratelimit_post_default(),
    }
  }
}

pub fn ratelimit_get_default() -> RatelimitConfig {
  RatelimitConfig {
    period: RatelimitPeriod::PerMs(unsafe { NonZeroU64::new_unchecked(500) }),
    burst_size: unsafe { NonZeroU32::new_unchecked(50) },
  }
}

pub fn ratelimit_post_default() -> RatelimitConfig {
  RatelimitConfig {
    period: RatelimitPeriod::PerSecond(unsafe { NonZeroU64::new_unchecked(1) }),
    burst_size: unsafe { NonZeroU32::new_unchecked(20) },
  }
}

impl Default for PowConfig {
  fn default() -> Self {
    Self {
      enabled: pow_enabled_default(),
      salt_size: pow_salt_size_default(),
      cost: pow_cost_default(),
      timestamp_delta: pow_timestamp_delta_default(),
    }
  }
}

#[inline]
pub fn pow_enabled_default() -> bool {
  true
}

#[inline]
pub fn pow_salt_size_default() -> NonZeroUsize {
  unsafe { NonZeroUsize::new_unchecked(32) }
}

#[inline]
pub fn pow_cost_default() -> u32 {
  19
}

#[inline]
pub fn pow_timestamp_delta_default() -> u64 {
  60
}
