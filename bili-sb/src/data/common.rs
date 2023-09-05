use std::fmt;

use axum::{response::IntoResponse, Json};
use serde::Serialize;

#[derive(Serialize, Default)]
pub struct Resp<T: Serialize> {
  pub code: RespCode,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub message: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub data: Option<T>,
}

impl<T: Serialize> IntoResponse for Resp<T> {
  #[inline(always)]
  fn into_response(self) -> axum::response::Response {
    Json(self).into_response()
  }
}

#[repr(transparent)]
#[derive(Clone, Copy, Serialize, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct RespCode(u32);

impl fmt::Debug for RespCode {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    fmt::Debug::fmt(&self.0, f)
  }
}

#[allow(dead_code)]
impl RespCode {
  #[inline]
  const fn success(self) -> bool {
    self.0 == 0
  }

  const fn from_u32(value: u32) -> Self {
    Self(value)
  }

  #[inline]
  const fn failure(self) -> bool {
    self.0 != 0
  }

  #[inline]
  const fn as_u32(self) -> u32 {
    self.0
  }
}

macro_rules! resp_codes {
  (
    $(
      ( $num:expr, $name:ident $(,)? )
    ),+
    $(,)?
  ) => {
    impl RespCode {
      $(
      pub const $name: RespCode = RespCode($num);
      )+

      pub fn describe(self) -> Option<&'static str> {
        match self.0 {
          $(
            $num => Some(stringify!($name)),
          )+
          _ => None
        }
      }

    }
  }
}

resp_codes! {
  (0, SUCCESS),
  (1, INVALID_PARAMS),
  (100, DATABASE_ERROR),
  (101, BILI_CLIENT_ERROR),
  (10000, UNKNOWN),
}

impl From<u32> for RespCode {
  #[inline]
  fn from(value: u32) -> Self {
    RespCode(value)
  }
}

impl<T: Serialize> Resp<T> {
  pub fn new_success(data: T) -> Resp<T> {
    Resp {
      code: RespCode::SUCCESS,
      message: None,
      data: Some(data),
    }
  }

  pub fn new_failure(code: RespCode, message: String) -> Resp<T> {
    Resp {
      code,
      message: Some(message),
      data: None,
    }
  }
}

impl<T: Serialize> From<T> for Resp<T> {
  #[inline(always)]
  fn from(value: T) -> Self {
    Self::new_success(value)
  }
}
