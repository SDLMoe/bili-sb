use std::time::SystemTime;

use diesel::pg::Pg;
use diesel::prelude::*;
use ipnet::IpNet;
use uuid::Uuid;

#[rustfmt::skip]
mod schema;

pub use schema::*;

#[derive(Queryable, Selectable)]
#[diesel(table_name = videos)]
#[diesel(check_for_backend(Pg))]
pub struct Video {
  pub aid: i64,
  pub title: String,
  pub update_time: SystemTime,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = video_parts)]
#[diesel(check_for_backend(Pg))]
pub struct VideoPart {
  pub aid: i64,
  pub cid: i64,
  pub title: String,
  pub duration: f32,
}

#[derive(Insertable, Queryable, Selectable)]
#[diesel(table_name = users)]
#[diesel(check_for_backend(Pg))]
pub struct User {
  pub id: Uuid,
  pub register_time: SystemTime,
  pub register_ip: IpNet,
  pub last_operation_time: Option<SystemTime>,
}

impl User {
  pub fn new(ip: IpNet) -> User {
    Self {
      id: Uuid::new_v4(),
      register_time: SystemTime::now(),
      register_ip: ip,
      last_operation_time: None,
    }
  }
}
