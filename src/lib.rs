pub mod common;

#[cfg(target_os = "android")]
#[allow(non_snake_case)]
pub mod android;

#[cfg(target_os = "ios")]
pub mod ios;
