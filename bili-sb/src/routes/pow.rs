use super::prelude::*;

#[derive(Default, Serialize)]
pub struct PowProblemData {
  pub enabled: bool,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub uuid: Option<Uuid>,
  /// base64-encoded salt
  #[serde(skip_serializing_if = "Option::is_none")]
  pub salt: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub cost: Option<u32>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub timestamp: Option<u64>,
}

pub async fn pow_choose(state: AppState) -> AppResult<Resp<PowProblemData>> {
  let config = &state.config.pow;
  if !config.enabled {
    let data = PowProblemData {
      enabled: false,
      ..Default::default()
    };
    return Ok(data.into());
  }

  let mut salt = vec![0; config.salt_size.get()];
  let cost = config.cost;
  let timestamp = blake3_pow::epoch_sec();
  rand::thread_rng().fill_bytes(&mut salt);

  let uuid = Uuid::new_v4();
  let data = PowProblemData {
    enabled: true,
    salt: Some(base64_simd::STANDARD.encode_to_string(&salt)),
    cost: Some(cost),
    timestamp: Some(timestamp),
    uuid: Some(uuid),
  };

  state.pow_map.insert(
    uuid,
    PowProperty {
      salt,
      cost,
      timestamp,
    },
  );

  spawn(async move {
    sleep(Duration::from_secs(60)).await;
    state.pow_map.remove(&uuid);
  });

  Ok(data.into())
}
