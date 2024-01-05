#[cfg(target_os = "ios")]
extern crate objc;

#[cfg(target_os = "ios")]
use objc::runtime::Object;
#[cfg(target_os = "ios")]
use objc::{class, msg_send, sel, sel_impl};

#[repr(C)]
struct UIEdgeInsets {
    top: f64,
    left: f64,
    bottom: f64,
    right: f64,
}

#[cfg(target_os = "ios")]
pub fn get_safe_area_insets() -> (f64, f64, f64, f64) {
    unsafe {
        let ui_application: *mut Object = msg_send![class!(UIApplication), sharedApplication];
        let key_window: *mut Object = msg_send![ui_application, keyWindow];
        let safe_area_insets: UIEdgeInsets = msg_send![key_window, safeAreaInsets];
        let top = safe_area_insets.top as f64;
        let bottom = safe_area_insets.bottom as f64;
        let left = safe_area_insets.left as f64;
        let right = safe_area_insets.right as f64;

        (top, bottom, left, right)
    }
}
