use std::{fs, path::PathBuf, str::FromStr};

use anyhow::Context;

fn main() -> anyhow::Result<()> {
  let protos: Vec<_> = glob::glob("proto/bilibili/**/*.proto")
    .context("Failed to glob protobuf files")?
    .filter_map(|file| file.ok())
    .collect();

  let output_proto =
    PathBuf::from_str(&std::env::var("OUT_DIR").context("No such env var `OUT_DIR`")?)
      .context("Cannot create pathbuf")?
      .join("proto");

  fs::create_dir_all(&output_proto)
    .with_context(|| format!("Cannot create directory {:?}", output_proto.to_str()))?;

  tonic_build::configure()
    .out_dir(output_proto)
    .build_server(false)
    .include_file("bilibili.rs")
    .compile(&protos, &["proto"])
    .context("Failed to compile protos")?;

  Ok(())
}
