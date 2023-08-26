use std::time::SystemTime;

use rand::Rng;

pub fn epoch_sec() -> u64 {
  SystemTime::now()
    .duration_since(SystemTime::UNIX_EPOCH)
    .unwrap()
    .as_secs()
}

pub fn verify(salt: &[u8], cost: u32, timestamp: u64, key: u128) -> bool {
  let now_ts = epoch_sec();
  if (now_ts.wrapping_add(60)..=now_ts.wrapping_sub(60)).contains(&timestamp) {
    return false;
  }

  let mut hasher = blake3::Hasher::new();
  hasher.update(salt);
  hasher.update(&timestamp.to_be_bytes());
  hasher.update(&key.to_be_bytes());
  let hash = hasher.finalize();
  hash.as_bytes().leading_zeros() >= cost
}

pub fn search(salt: &[u8], cost: u32, timestamp: u64, max_effort: usize) -> Option<u128> {
  let mut rng = rand::thread_rng();
  for _ in 0..max_effort {
    let key: u128 = rng.gen();
    let mut hasher = blake3::Hasher::new();
    hasher.update(salt);
    hasher.update(&timestamp.to_be_bytes());
    hasher.update(&key.to_be_bytes());
    let hash = hasher.finalize();
    if hash.as_bytes().leading_zeros() >= cost {
      return Some(key);
    }
  }
  None
}

trait LeadingZeros {
  fn leading_zeros(&self) -> u32;
}

impl LeadingZeros for [u8] {
  fn leading_zeros(&self) -> u32 {
    let mut zeros = 0;
    for &byte in self {
      zeros += byte.leading_zeros();
      if byte != 0 {
        break;
      }
    }
    zeros
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn leading_zeros() {
    let u64 = u64::from_be_bytes([0, 0, 0, 0, 0, 0, 0, 1]);
    assert_eq!(u64, 1);
    assert_eq!(u64.to_be_bytes().leading_zeros(), u64.leading_zeros(),);
  }
}
