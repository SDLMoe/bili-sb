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
    R::Cids { cids } => {
      // SAFETY: NonZeroU64 even can be *cast* to i64, so it's safe to transmute it
      let mut db_con = state.db_con().await?;
      let cids: Vec<i64> = unsafe { transmute(cids) };

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
