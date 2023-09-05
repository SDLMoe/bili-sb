use std::num::NonZeroU64;

use serde::{Deserialize, Serialize};

mod abv;
mod common;

pub use crate::data::abv::*;
pub use common::*;
use uuid::Uuid;

#[derive(Serialize)]
pub struct CreateUserData {
  pub uuid: Uuid,
}

#[derive(Serialize)]
pub struct PowProblemData {
  pub uuid: Uuid,
  /// base64-encoded salt
  pub salt: String,
  pub cost: u32,
  pub timestamp: u64,
}

#[derive(Deserialize, Debug)]
pub struct CreateSegmentReq {
  pub start: f32,
  pub end: f32,
  #[serde(flatten)]
  pub abv: Abv,
  pub cid: NonZeroU64,
  pub submitter: Uuid,
}
