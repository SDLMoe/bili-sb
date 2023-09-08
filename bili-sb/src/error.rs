use std::{error::Error as StdError, fmt::Display, ops::Deref};

use anyhow::Context;
use axum::response::{Html, IntoResponse};
use bili_proto::bilibili::rpc::Status as BiliStatus;
use diesel::result::Error as DieselError;
use http::{StatusCode, Uri};
use indoc::formatdoc;
use prost::Message;

use crate::data::{Resp, RespCode};

pub type AppResult<T, E = AnyhowWrapper> = Result<T, E>;

#[repr(transparent)]
#[must_use]
pub struct AnyhowWrapper(pub anyhow::Error);

impl AnyhowWrapper {
  #[inline]
  pub fn new(inner: anyhow::Error) -> Self {
    Self(inner)
  }

  #[inline]
  pub fn new_inner<E>(inner: E) -> Self
  where
    E: StdError + Send + Sync + 'static,
  {
    Self(anyhow::Error::new(inner))
  }
}

impl From<anyhow::Error> for AnyhowWrapper {
  fn from(value: anyhow::Error) -> Self {
    AnyhowWrapper(value)
  }
}

impl From<AnyhowWrapper> for anyhow::Error {
  fn from(val: AnyhowWrapper) -> Self {
    val.0
  }
}

impl Deref for AnyhowWrapper {
  type Target = anyhow::Error;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl IntoResponse for AnyhowWrapper {
  fn into_response(self) -> axum::response::Response {
    let Some(app_error) = self.0.downcast_ref::<AppError>() else {
      log::error!("Unexpected Error: {:?}", self.0);
      let json = Resp::<()>::new_failure(RespCode::UNKNOWN, "UNKNOWN".to_string());
      return (StatusCode::INTERNAL_SERVER_ERROR, json).into_response();
    };

    if app_error.http_code.is_server_error() {
      log::error!("Unexpected Error: {:?}", self.0);
    }

    let resp = Resp::<()>::new_failure(app_error.resp_code, format!("{:?}", self.0));
    (app_error.http_code, resp).into_response()
  }
}

#[derive(thiserror::Error, Debug)]
pub struct AppError {
  pub http_code: StatusCode,
  pub resp_code: RespCode,
}

impl Display for AppError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.resp_code.describe().unwrap_or("Unknown"))
  }
}

impl Default for AppError {
  #[inline]
  fn default() -> Self {
    Self::new()
  }
}

#[allow(dead_code)]
impl AppError {
  pub fn new() -> AppError {
    AppError {
      http_code: StatusCode::INTERNAL_SERVER_ERROR,
      resp_code: RespCode::UNKNOWN,
    }
  }

  #[inline]
  pub fn http_code(mut self, code: StatusCode) -> Self {
    self.http_code = code;
    self
  }

  #[inline]
  pub fn resp_code(mut self, code: RespCode) -> Self {
    self.resp_code = code;
    self
  }
}

pub trait AnyhowExt<T> {
  fn with_app_error(self, code: RespCode) -> Self;
}

impl<T> AnyhowExt<T> for anyhow::Result<T> {
  #[inline]
  fn with_app_error(self, code: RespCode) -> Self {
    self.context(AppError::new().resp_code(code))
  }
}

pub trait IntoAppResult<T> {
  fn into_app_result(self) -> AppResult<T>;

  fn context_into_app<C>(self, context: C) -> AppResult<T>
  where
    C: Display + Send + Sync + 'static;

  fn with_context_into_app<C, F>(self, context: F) -> AppResult<T>
  where
    C: Display + Send + Sync + 'static,
    F: FnOnce() -> C;
}

impl<T> IntoAppResult<T> for anyhow::Result<T> {
  #[inline]
  fn into_app_result(self) -> AppResult<T> {
    match self {
      Ok(ok) => Ok(ok),
      Err(err) => Err(AnyhowWrapper(err)),
    }
  }

  #[inline]
  fn context_into_app<C>(self, context: C) -> AppResult<T>
  where
    C: Display + Send + Sync + 'static,
  {
    match self {
      Ok(t) => Ok(t),
      Err(e) => Err(AnyhowWrapper(e.context(context))),
    }
  }

  #[inline]
  fn with_context_into_app<C, F>(self, context: F) -> AppResult<T>
  where
    C: Display + Send + Sync + 'static,
    F: FnOnce() -> C,
  {
    match self {
      Ok(ok) => Ok(ok),
      Err(error) => Err(AnyhowWrapper(error.context(context()))),
    }
  }
}

macro_rules! impl_into_app {
  () => {
    #[inline]
    fn into_app_result(self) -> AppResult<T> {
      match self {
        Ok(ok) => Ok(ok),
        Err(err) => Err(AnyhowWrapper(anyhow::Error::new(err))),
      }
    }
  };
}

impl<T> IntoAppResult<T> for Result<T, tonic::Status> {
  impl_into_app!();

  fn context_into_app<C>(self, context: C) -> AppResult<T>
  where
    C: Display + Send + Sync + 'static,
  {
    self
      .map_err(map_bili_status)
      .map_err(anyhow::Error::new)
      .with_app_error(RespCode::BILI_CLIENT_ERROR)
      .context(context)
      .into_app_result()
  }

  fn with_context_into_app<C, F>(self, context: F) -> AppResult<T>
  where
    C: Display + Send + Sync + 'static,
    F: FnOnce() -> C,
  {
    self
      .map_err(map_bili_status)
      .map_err(anyhow::Error::new)
      .with_context(context)
      .with_app_error(RespCode::BILI_CLIENT_ERROR)
      .into_app_result()
  }
}

impl<T> IntoAppResult<T> for Result<T, DieselError> {
  impl_into_app!();

  fn context_into_app<C>(self, context: C) -> AppResult<T>
  where
    C: Display + Send + Sync + 'static,
  {
    self
      .context(context)
      .with_app_error(RespCode::DATABASE_ERROR)
      .into_app_result()
  }

  fn with_context_into_app<C, F>(self, context: F) -> AppResult<T>
  where
    C: Display + Send + Sync + 'static,
    F: FnOnce() -> C,
  {
    self
      .with_context(context)
      .with_app_error(RespCode::DATABASE_ERROR)
      .into_app_result()
  }
}

#[derive(Debug)]
pub struct RpcStatus {
  pub code: i32,
  pub message: String,
}

#[derive(Debug, thiserror::Error)]
pub enum RpcError {
  #[error("{self:?}")]
  Raw(tonic::Status),
  #[error("{self:?}")]
  Parsed(Vec<RpcStatus>),
}

fn map_bili_status(raw: tonic::Status) -> RpcError {
  let Some(parsed) = BiliStatus::decode(raw.details()).ok() else {
    return RpcError::Raw(raw);
  };
  let mut parsed_vec = Vec::with_capacity(1 + parsed.details.len());
  parsed_vec.push(RpcStatus {
    code: parsed.code,
    message: parsed.message,
  });

  for any in parsed.details {
    if !any.type_url.ends_with("bilibili.rpc.Status") {
      return RpcError::Raw(raw);
    }
    let status = BiliStatus::decode(any.value.as_slice()).ok().unwrap();
    if !status.details.is_empty() {
      return RpcError::Raw(raw);
    }
    parsed_vec.push(RpcStatus {
      code: status.code,
      message: status.message,
    });
  }

  RpcError::Parsed(parsed_vec)
}

pub fn error_html(uri: Option<&Uri>, code: StatusCode, extra_msg: &str) -> Html<String> {
  let reason = code.canonical_reason().unwrap_or("");
  let code = code.as_u16();
  let path = uri
    .map(|uri| {
      formatdoc! {
        r#"
      <tr>
      <td>Path:</td>
      <td>{path}</td>
      </tr>"#,
        path = uri.path()
      }
    })
    .unwrap_or_else(|| "".to_string());

  Html(formatdoc! {
    r##"
    <!DOCTYPE HTML PUBLIC "-//IETF//DTD HTML 2.0//EN">
    <html>
    <head><title>{code} {reason}</title></head>
    <body>
    <center><h1>{code} {reason}</h1></center>
    Sorry for the inconvenience.<br/>
    Please <a href="{homepage}/issues/new">submit a issue</a> and include the following information to us.<br/>
    Thank you very much!<br/>
    </p>
    <table>{path_entry}
    <tr>
    <td>Reason:</td>
    <td><pre>{extra_msg}</pre></td>
    </tr>
    </table>
    <hr/>Powered by <a href="{homepage}">bili-sb</a><hr><center>bili-sb</center>
    </body>
    </html>
    "##,
    path_entry = path,
    extra_msg = html_escape::encode_safe(extra_msg),
    homepage = env!("CARGO_PKG_HOMEPAGE"),
  })
}
