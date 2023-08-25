include_proto!(bilibili);

macro_rules! include_proto {
  ( $($package:ident).+ ) => {
      include!(concat!(env!("OUT_DIR"), "/proto/", stringify!($($package).+), ".rs"));
  };
  ( $package:literal ) => {
      include!(concat!(env!("OUT_DIR"), "/proto/", $package, ".rs"));
  };
}
pub(crate) use include_proto;
