use std::time::SystemTime;

use super::prelude::*;

use bilibili::app::archive::v1::Arc as Archive;

/// Returns AppResult<Resp<db::Segment>>
///
/// See also: [db::Segment]
pub async fn segment_create(
  state: AppState,
  ip: SecureClientIp,
  body: Json<CreateSegmentReq>,
) -> AppResult<Response> {
  let bili = state.bili().await?;
  let mut view = pb_client!(bili, ViewClient);
  let mut db_con: PooledPgCon = state.db_con_owned().await?;
  let users: Vec<db::User> = db::users::table
    .filter(db::users::id.eq(body.submitter))
    .load(&mut db_con)
    .await
    .context_into_app("Failed to fetch user")?;

  if body.start > body.end {
    return Err(app_err_custom!(
      StatusCode::UNPROCESSABLE_ENTITY,
      RespCode::INVALID_PARAMS,
      "segment start is larger than end, {} > {}",
      body.start,
      body.end
    ));
  };

  let Some(user) = users.get(0).cloned() else {
    return Err(app_err_custom!(
      StatusCode::UNPROCESSABLE_ENTITY,
      RespCode::INVALID_PARAMS,
      "Invalid user uuid `{}`",
      body.submitter
    ));
  };

  let reply = view
    .view(view::ViewReq {
      aid: body.abv.as_i64(),
      ..Default::default()
    })
    .await
    .with_context_into_app(|| format!("Unable to fetch video aid `{}`", body.abv.av()))?
    .into_inner();

  let archive: Archive = reply
    .arc
    .context("ViewReply malformed, no `arc` field")
    .with_app_error(RespCode::BILI_CLIENT_ERROR)?;

  let Some(aid) = Abv::new(archive.aid as u64) else {
    return Err(app_err_custom!(
      StatusCode::UNPROCESSABLE_ENTITY,
      RespCode::BILI_CLIENT_ERROR,
      "ViewReply malformed, aid == 0"
    ));
  };

  let mut parts = Vec::with_capacity(reply.pages.len());

  for page in reply.pages {
    let page = page
      .page
      .context("ViewPage malformed, no `page` field")
      .with_app_error(RespCode::BILI_CLIENT_ERROR)?;

    if page.cid == 0 {
      return Err(app_err!(
        RespCode::BILI_CLIENT_ERROR,
        "ViewPage.Page malformed, cid == 0"
      ));
    };

    parts.push(db::VideoPart {
      aid: aid.as_i64(),
      cid: page.cid,
      title: page.part,
      duration: page.duration as f32,
    });
  }

  if !parts.iter().any(|page| page.cid == body.cid.get() as i64) {
    return Err(app_err_custom!(
      StatusCode::UNPROCESSABLE_ENTITY,
      RespCode::INVALID_PARAMS,
      "cid is not valid {}",
      body.cid
    ));
  }

  let video = db::Video {
    aid: aid.as_i64(),
    title: archive.title,
    update_time: SystemTime::now(),
  };

  let new_user = db::User {
    last_operation_time: Some(SystemTime::now()),
    ..user.clone()
  };

  let segment = Arc::new(db::Segment {
    id: Uuid::new_v4(),
    cid: body.cid.get() as i64,
    start: body.start,
    end: body.end,
    submitter: user.id,
    submitter_ip: ip.0.into(),
  });

  let db_segment = Arc::clone(&segment);
  spawn(async move {
    let update_result = db_con
      .build_transaction()
      .run::<_, diesel::result::Error, _>(|con| {
        async move {
          diesel::insert_into(db::users::table)
            .values(&new_user)
            .on_conflict(db::users::id)
            .do_update()
            .set(&new_user)
            .execute(con)
            .await?;

          diesel::insert_into(db::videos::table)
            .values(&video)
            .on_conflict(db::videos::aid)
            .do_update()
            .set(&video)
            .execute(con)
            .await?;

          for part in parts.iter() {
            diesel::insert_into(db::video_parts::table)
              .values(part)
              .on_conflict(db::video_parts::cid)
              .do_update()
              .set(part)
              .execute(con)
              .await?;
          }

          diesel::insert_into(db::segments::table)
            .values(db_segment.as_ref())
            .execute(con)
            .await?;

          Ok(())
        }
        .scope_boxed()
      })
      .await
      .with_context(|| format!("Failed to insert video (aid `{}`) and its parts", aid.av()));

    if let Err(err) = update_result {
      error!("{:?}", err);
    }
  });

  Ok(Resp::new_success(segment.as_ref()).into_response())
}
