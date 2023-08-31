use axum::{
  middleware::Next,
  response::{IntoResponse, Response},
};
use http::{Method, Request, StatusCode};

use crate::state::*;

pub const POW_HEADER_UUID: &str = "bilisb-pow-uuid";
pub const POW_HEADER_SOLUTION: &str = "bilisb-pow-solution";

pub async fn pow_layer<B>(state: AppState, mut request: Request<B>, next: Next<B>) -> Response {
  if request.method() != Method::POST {
    return next.run(request).await;
  }
  if request.uri().path().starts_with("/pow/choose") {
    return next.run(request).await;
  }

  let Some(uuid) = request
    .headers_mut()
    .remove(POW_HEADER_UUID)
    .and_then(|value| uuid::Uuid::try_parse_ascii(value.as_bytes()).ok())
  else {
    return (
      StatusCode::BAD_REQUEST,
      "header `bilisb-pow-uuid` does not exist or malformed",
    )
      .into_response();
  };

  let Some(solution) = request
    .headers_mut()
    .remove(POW_HEADER_SOLUTION)
    .and_then(|value| value.to_str().ok()?.parse::<u128>().ok())
  else {
    return (
      StatusCode::BAD_REQUEST,
      "header `bilisb-pow-solution` does not exist or malformed",
    )
      .into_response();
  };

  let Some((_, pow)) = state.pow_map.remove(&uuid) else {
    return (
      StatusCode::BAD_REQUEST,
      format!("uuid `{uuid}` invalid or expired"),
    )
      .into_response();
  };

  if !blake3_pow::verify(&pow.salt, pow.cost, pow.timestamp, solution) {
    return (StatusCode::BAD_REQUEST, "wrong answer".to_string()).into_response();
  }

  next.run(request).await
}
