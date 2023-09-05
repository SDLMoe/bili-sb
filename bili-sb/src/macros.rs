#[macro_export]
macro_rules! app_err {
  ($resp_code:expr, $msg:literal $(,)?) => {{
    $crate::error::AnyhowWrapper(
      ::anyhow::anyhow!($msg).context($crate::error::AppError::new().resp_code($resp_code))
    )
  }};
  ($resp_code:expr, $fmt:expr, $($arg:tt)*) => {{
    $crate::error::AnyhowWrapper(
      ::anyhow::anyhow!($fmt, $($arg)*).context($crate::error::AppError::new().resp_code($resp_code))
    )
  }};
}

#[macro_export]
macro_rules! ratelimit {
  ( $conf:expr $(,)? ) => {
    ::tower::ServiceBuilder::new()
      .layer(::axum::error_handling::HandleErrorLayer::new(
        |e| async move { tower_governor::errors::display_error(e) },
      ))
      .layer(::tower_governor::GovernorLayer { config: $conf })
  };
}
