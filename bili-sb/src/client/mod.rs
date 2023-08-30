//! Bilibili client for fetching video metadata

use anyhow::Context;
pub use bili_proto::bilibili::{
  self,
  app::view::v1::{self as view, view_client::ViewClient},
};
use once_cell::sync::Lazy;
use prost::Message;
use tonic::{metadata::MetadataValue, transport::Channel, Request, Status};

/// Usage:
///
/// ```no_run
///  let channel = connect(BILI_GRPC_URL).await?;
///  let mut foo = pb_client!(channel.clone(), FooClient);
///  let mut bar = pb_client!(channel, BarClient);
/// ```
#[macro_export]
macro_rules! pb_client {
  ($channel:expr, $client:ident $(,)?) => {
    <$client<::tonic::transport::Channel>>::with_interceptor(
      $channel,
      $crate::client::bili_interceptor,
    )
  };
}

pub const BILI_GRPC_URL: &str = "https://grpc.biliapi.net";
pub const BILI_GRPC_FAILOVER_URL: &str = "https://app.bilibili.com";

pub fn bili_interceptor(request: Request<()>) -> Result<Request<()>, Status> {
  let (mut meta, exts, msg) = request.into_parts();
  static METADATA: Lazy<&'static str> = Lazy::new(|| {
    let metadata = bilibili::metadata::Metadata::default().encode_to_vec();
    let encoded: &'static [u8] = MetadataValue::from_bytes(&metadata)
      .as_encoded_bytes()
      .to_vec()
      .leak();
    unsafe { std::str::from_utf8_unchecked(encoded) }
  });
  meta.insert_bin("x-bili-metadata-bin", MetadataValue::from_static(*METADATA));
  Ok(Request::from_parts(meta, exts, msg))
}

pub async fn connect(uri: &str) -> anyhow::Result<Channel> {
  Channel::builder(uri.parse().context("Failed to parse string as uri")?)
    .connect()
    .await
    .with_context(|| format!("Failed to connect server `{}`", uri))
}
