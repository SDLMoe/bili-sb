mod segment_create;
mod segment_list;

pub use self::segment_create::*;
pub use self::segment_list::*;

/// Prelude for `routes` mod
mod prelude {
  pub use anyhow::Context;
  pub use axum::Json;
  pub use diesel::{ExpressionMethods, QueryDsl};
  pub use diesel_async::{scoped_futures::ScopedFutureExt, RunQueryDsl};
  pub use log::{debug, error, info, warn};
  pub use tokio::spawn;

  pub use crate::{client::*, error::*, state::*, *};
}
