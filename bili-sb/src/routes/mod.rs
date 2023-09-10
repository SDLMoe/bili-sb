mod pow;
mod segment_create;
mod segment_list;
mod segment_vote;
mod user_create;

pub use pow::*;
pub use segment_create::*;
pub use segment_list::*;
pub use segment_vote::*;
pub use user_create::*;

/// Prelude for `routes` mod
mod prelude {
  pub use anyhow::Context;
  pub use axum::Json;
  pub use diesel::{ExpressionMethods, QueryDsl};
  pub use diesel_async::{scoped_futures::ScopedFutureExt, RunQueryDsl};
  pub use log::{debug, error, info, warn};
  pub use serde::{Deserialize, Serialize};
  pub use tokio::spawn;

  pub use crate::{client::*, error::*, state::*, *};
}
