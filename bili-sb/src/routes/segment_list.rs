use std::mem::transmute;

use diesel::SelectableHelper;

use super::prelude::*;

pub async fn segment_list(
  state: AppState,
  body: Json<ListSegmentReq>,
) -> AppResult<Resp<ListSegmentData>> {
  use ListSegmentReq as R;

  let segments: Vec<db::Segment> = match body.0 {
    R::Abv { abv } => {
      let mut db_con = state.db_con().await?;
      let aid = abv.as_i64();

      let vec: Vec<db::Segment> = db::video_parts::table
        .limit(100)
        .inner_join(db::segments::table)
        .select(db::Segment::as_select())
        .load::<db::Segment>(&mut db_con)
        .await
        .with_context_into_app(|| format!("Failed to fetch segments for aid {aid}"))?;

      vec
    },
    R::Cid { cid } => {
      let mut db_con = state.db_con().await?;
      let cid = cid.get() as i64;

      let vec: Vec<db::Segment> = db::segments::table
        .limit(100)
        .select(db::Segment::as_select())
        .filter(db::segments::cid.eq(cid))
        .load::<db::Segment>(&mut db_con)
        .await
        .with_context_into_app(|| format!("Failed to fetch segments for cid {cid}"))?;

      vec
    },
    R::Cids { mut cids } => {
      let mut db_con = state.db_con().await?;
      // # SAFETY
      //   - NonZeroU64 is `#[repr(transparent)]` for u64,
      //   - Casting between `u64` and `i64` is a no-op
      //   - But the field order of `Vec` is not guaranteed, so we need `Vec::from_raw_parts` here
      //
      // # See also
      //   - https://doc.rust-lang.org/nomicon/transmutes.html
      //     (`Vec<i32>` and `Vec<u32>` *might* have their fields in the same order, or they might not)
      //   - https://doc.rust-lang.org/reference/expressions/operator-expr.html#semantics
      //   - https://doc.rust-lang.org/stable/std/num/struct.NonZeroU64.html#layout-1
      let cids: Vec<i64> =
        unsafe { Vec::from_raw_parts(transmute(cids.as_mut_ptr()), cids.len(), cids.capacity()) };

      let vec: Vec<db::Segment> = db::segments::table
        .limit(100)
        .select(db::Segment::as_select())
        .filter(db::segments::cid.eq_any(&cids))
        .load::<db::Segment>(&mut db_con)
        .await
        .with_context_into_app(|| {
          format!(
            "Failed to fetch segment for cids {:?} (truncated)",
            cids.into_iter().take(5).collect::<Box<[_]>>()
          )
        })?;

      vec
    },
  };

  Ok(
    ListSegmentData {
      len: segments.len(),
      segments,
    }
    .into(),
  )
}
