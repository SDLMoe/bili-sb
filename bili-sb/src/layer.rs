use std::net::IpAddr;

use axum::{
  middleware::Next,
  response::{IntoResponse, Response},
};
use axum_client_ip::*;
use http::{Method, Request, StatusCode};
use tower_governor::{key_extractor::KeyExtractor, GovernorError};

use crate::state::*;

pub const POW_HEADER_UUID: &str = "bilisb-pow-uuid";
pub const POW_HEADER_SOLUTION: &str = "bilisb-pow-solution";

pub async fn pow_layer<B>(state: AppState, mut request: Request<B>, next: Next<B>) -> Response {
  let config = &state.config.pow;
  if !config.enabled {
    return next.run(request).await;
  }

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

  if !blake3_pow::verify(
    &pow.salt,
    pow.cost,
    pow.timestamp,
    config.timestamp_delta,
    solution,
  ) {
    return (StatusCode::BAD_REQUEST, "wrong answer".to_string()).into_response();
  }

  next.run(request).await
}

#[derive(Clone, Debug)]
pub struct SecureIpExtractor;

impl KeyExtractor for SecureIpExtractor {
  type Key = IpAddr;

  fn extract<T>(&self, req: &Request<T>) -> Result<Self::Key, GovernorError> {
    req.extensions();
    SecureClientIp::from(
      req
        .extensions()
        .get()
        .ok_or_else(|| GovernorError::UnableToExtractKey)?,
      req.headers(),
      req.extensions(),
    )
    .map(|ip| ip.0)
    .map_err(|_err| GovernorError::UnableToExtractKey)
  }
}
