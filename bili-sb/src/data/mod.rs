use serde::Serialize;

mod common;

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
