use std::num::NonZeroU64;

use serde::{Deserialize, Serialize};

mod abv;
mod common;

pub use crate::data::abv::*;
use crate::db;
pub use common::*;
use uuid::Uuid;

#[derive(Serialize)]
pub struct CreateUserData {
  pub uuid: Uuid,
}

#[derive(Default, Serialize)]
pub struct PowProblemData {
  pub enabled: bool,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub uuid: Option<Uuid>,
  /// base64-encoded salt
  #[serde(skip_serializing_if = "Option::is_none")]
  pub salt: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub cost: Option<u32>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub timestamp: Option<u64>,
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

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum ListSegmentReq {
  /// aid or bvid, lookup related cids for video
  Abv {
    #[serde(flatten)]
    abv: Abv,
  },
  /// single cid
  Cid { cid: NonZeroU64 },
  /// batch cids
  Cids { cids: Vec<NonZeroU64> },
}

#[derive(Serialize, Debug, Clone)]
pub struct ListSegmentData {
  pub len: usize,
  pub segments: Vec<db::Segment>,
}
