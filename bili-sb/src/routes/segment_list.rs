use std::{mem::transmute, num::NonZeroU64};

use super::prelude::*;

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
  pub segments: Vec<db::SegmentWithVote>,
}

pub async fn segment_list(
  state: AppState,
  body: Json<ListSegmentReq>,
) -> AppResult<Resp<ListSegmentData>> {
  use ListSegmentReq as R;

  let segments: Vec<db::SegmentWithVote> = match body.0 {
    R::Abv { abv } => {
      let mut db_con = state.db_con().await?;
      let aid = abv.as_i64();

      db::segments_related_to_aid(&mut db_con, aid)
        .await
        .with_context_into_app(|| format!("Failed to fetch segments for aid {aid}"))?
    },
    R::Cid { cid } => {
      let mut db_con = state.db_con().await?;
      let cid = cid.get() as i64;

      db::segments_related_to_cid(&mut db_con, cid)
        .await
        .with_context_into_app(|| format!("Failed to fetch segments for cid {cid}"))?
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

      db::segments_related_to_cids(&mut db_con, &cids)
        .await
        .with_context_into_app(|| {
          format!(
            "Failed to fetch segment for cids {:?} (truncated)",
            cids.into_iter().take(5).collect::<Box<[_]>>()
          )
        })?
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
