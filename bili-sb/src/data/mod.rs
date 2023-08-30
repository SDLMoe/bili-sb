use serde::Serialize;

mod common;

pub use common::*;
use uuid::Uuid;

#[derive(Serialize)]
pub struct PagesData {
  pub pages: Vec<u64>,
}

#[derive(Serialize)]
pub struct CreateUserData {
  pub uuid: Uuid,
}
