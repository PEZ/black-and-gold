#[cfg(target_os = "ios")]
extern crate objc;

#[cfg(target_os = "ios")]
use objc::runtime::Object;
#[cfg(target_os = "ios")]
use objc::{class, msg_send, sel, sel_impl};
