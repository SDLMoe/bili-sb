use std::time::SystemTime;

use diesel::pg::Pg;
use diesel::prelude::*;
use diesel_derive_enum::DbEnum;
use ipnet::IpNet;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[rustfmt::skip]
mod schema;

pub use schema::*;

#[derive(Debug, Insertable, Queryable, Selectable, AsChangeset)]
#[diesel(table_name = videos)]
#[diesel(check_for_backend(Pg))]
pub struct Video {
  pub aid: i64,
  pub title: String,
  pub update_time: SystemTime,
}

#[derive(Debug, Insertable, Queryable, Selectable, AsChangeset)]
#[diesel(table_name = video_parts)]
#[diesel(check_for_backend(Pg))]
pub struct VideoPart {
  pub aid: i64,
  pub cid: i64,
  pub title: String,
  pub duration: f32,
}

#[derive(Debug, Clone, Insertable, Queryable, Selectable, AsChangeset)]
#[diesel(table_name = users)]
#[diesel(check_for_backend(Pg))]
pub struct User {
  pub id: Uuid,
  pub register_time: SystemTime,
  pub register_ip: IpNet,
  pub last_operation_ip: Option<IpNet>,
  pub last_operation_time: Option<SystemTime>,
}

impl User {
  pub fn new(ip: IpNet) -> User {
    Self {
      id: Uuid::new_v4(),
      register_time: SystemTime::now(),
      register_ip: ip,
      last_operation_ip: None,
      last_operation_time: None,
    }
  }
}

#[derive(Serialize, Clone, Debug, Insertable, Queryable, Selectable)]
#[diesel(table_name = segments)]
#[diesel(check_for_backend(Pg))]
pub struct Segment {
  pub id: Uuid,
  pub cid: i64,
  pub start: f32,
  pub end: f32,
  #[serde(with = "humantime_serde")]
  pub time: SystemTime,
  #[serde(skip)]
  pub submitter: Uuid,
  #[serde(skip)]
  pub submitter_ip: IpNet,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, DbEnum)]
#[ExistingTypePath = "schema::sql_types::VoteType"]
#[serde(rename_all = "snake_case")]
pub enum VoteType {
  Up,
  Down,
}

impl Default for VoteType {
  fn default() -> Self {
    Self::Up
  }
}

#[derive(Clone, Debug, Insertable, Queryable, Selectable, AsChangeset)]
#[diesel(table_name = votes)]
#[diesel(check_for_backend(Pg))]
pub struct Vote {
  pub segment: Uuid,
  pub type_: VoteType,
  pub voter: Uuid,
  pub voter_ip: IpNet,
  pub time: SystemTime,
}
