use std::time::SystemTime;

use super::prelude::*;

#[derive(Deserialize, Debug)]
pub struct SegmentVoteReq {
  pub id: Uuid,
  pub voter: Uuid,
  #[serde(default)]
  pub r#type: db::VoteType,
}

#[derive(Serialize)]
pub struct SegmentVoteResp {
  pub up: i64,
  pub down: i64,
}

pub async fn segment_vote(
  state: AppState,
  ip: SecureClientIp,
  body: Json<SegmentVoteReq>,
) -> AppResult<Resp<SegmentVoteResp>> {
  let mut db_con: PooledPgCon = state.db_con().await?;
  let segment_exist = db::segments::table
    .filter(db::segments::id.eq(body.id))
    .count()
    .get_result::<i64>(&mut db_con)
    .await
    .with_context_into_app(|| format!("No such segment, uuid = {}", body.id))?
    == 1;

  if !segment_exist {
    return Err(app_err_custom!(
      StatusCode::UNPROCESSABLE_ENTITY,
      RespCode::INVALID_PARAMS,
      "No such segment, uuid = {}",
      body.id
    ));
  }

  let user_exist = db::users::table
    .filter(db::users::id.eq(body.voter))
    .count()
    .get_result::<i64>(&mut db_con)
    .await
    .with_context_into_app(|| format!("No such segment, uuid = {}", body.id))?
    == 1;

  if !user_exist {
    return Err(app_err_custom!(
      StatusCode::UNPROCESSABLE_ENTITY,
      RespCode::INVALID_PARAMS,
      "No such user, uuid = {}",
      body.voter
    ));
  }

  let vote = db::Vote {
    segment: body.id,
    type_: body.r#type,
    voter: body.voter,
    voter_ip: ip.0.into(),
    time: SystemTime::now(),
  };

  diesel::insert_into(db::votes::table)
    .values(&vote)
    .on_conflict((db::votes::segment, db::votes::voter))
    .do_update()
    .set(&vote)
    .execute(&mut db_con)
    .await
    .with_context_into_app(|| format!("Failed to upsert votes, request: {:?}", &body.0))?;

  let up_votes: i64 = db::count_votes(&mut db_con, body.id, db::VoteType::Up)
    .await
    .with_context_into_app(|| format!("Failed to get up vote count for request: {:?}", &body.0))?;

  let down_votes = db::count_votes(&mut db_con, body.id, db::VoteType::Down)
    .await
    .with_context_into_app(|| {
      format!("Failed to get down vote count for request: {:?}", &body.0)
    })?;

  Ok(
    SegmentVoteResp {
      up: up_votes,
      down: down_votes,
    }
    .into(),
  )
}
