use super::prelude::*;

#[derive(Serialize)]
pub struct CreateUserData {
  pub uuid: Uuid,
}

pub async fn user_create(state: AppState, ip: SecureClientIp) -> AppResult<Resp<CreateUserData>> {
  let mut con = state.db_con().await?;
  let user = db::User::new(ip.0.into());
  let result = diesel::insert_into(db::users::table)
    .values(&user)
    .execute(&mut con)
    .await
    .context_into_app("Failed to insert")?;

  if result != 1 {
    return Err(app_err!(RespCode::DATABASE_ERROR, "Database insert failed"));
  }

  Ok(CreateUserData { uuid: user.id }.into())
}
