use std::{borrow::Cow, num::NonZeroU64};

use serde::Deserialize;

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct Abv(NonZeroU64);

impl From<Abv> for NonZeroU64 {
  fn from(val: Abv) -> Self {
    val.0
  }
}

impl From<Abv> for u64 {
  fn from(val: Abv) -> Self {
    val.0.get()
  }
}

#[allow(dead_code)]
impl Abv {
  pub fn new(aid: u64) -> Option<Abv> {
    if !(abv::MIN_AID..abv::MAX_AID).contains(&aid) {
      return None;
    }
    Some(unsafe { Self::new_unchecked(aid) })
  }

  #[inline]
  pub unsafe fn new_unchecked(aid: u64) -> Abv {
    Abv(unsafe { NonZeroU64::new_unchecked(aid) })
  }

  pub fn av(self) -> u64 {
    self.0.get()
  }

  pub fn as_i64(self) -> i64 {
    self.0.get() as i64
  }

  pub fn bv(self) -> String {
    // SAFETY: `Abv` have guaranteed `self.0` must be within the range
    #[cfg(not(debug_assertions))]
    unsafe {
      abv::av2bv(self.0.get()).unwrap_unchecked()
    }
    #[cfg(debug_assertions)]
    abv::av2bv(self.0.get()).unwrap()
  }
}

impl<'de> Deserialize<'de> for Abv {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    #[derive(Deserialize, Debug)]
    #[serde(rename_all = "snake_case")]
    enum AbvInner<'a> {
      Aid(u64),
      Bvid(#[serde(borrow)] Cow<'a, str>),
    }

    let abv = AbvInner::deserialize(deserializer)?;
    match abv {
      AbvInner::Aid(aid) => match Abv::new(aid) {
        Some(av) => Ok(av),
        None => Err(serde::de::Error::custom("invalid aid, number out of range")),
      },
      AbvInner::Bvid(bvid) => match abv::bv2av(bvid) {
        Ok(inner) => unsafe { Ok(Abv::new_unchecked(inner)) },
        Err(error) => Err(serde::de::Error::custom(format!(
          "Unexpected invalid bvid, {error:?}"
        ))),
      },
    }
  }
}

#[test]
fn abv_test() {
  use super::*;
  use serde_json::json;

  #[derive(Deserialize)]
  struct Test {
    #[serde(flatten)]
    abv: Abv,
  }

  let json = json!({"bvid": "BV1Gb4y1C78H"});
  let data: Test = serde_json::from_value(json).unwrap();
  assert_eq!(data.abv.av(), 631295196);

  let json = json!({"aid": 170001});
  let data: Test = serde_json::from_value(json).unwrap();
  assert_eq!(data.abv.av(), 170001);

  let json = json!({"aid": 0});
  let data = serde_json::from_value::<Test>(json);
  assert!(data.is_err());

  let json = json!({"bvid": "BV0asdfas"});
  let data = serde_json::from_value::<Test>(json);
  assert!(data.is_err());
}
