use std::fmt;

#[cfg(target_os = "ios")]
extern crate objc;

#[cfg(target_os = "ios")]
use objc::runtime::Object;
#[cfg(target_os = "ios")]
use objc::{class, msg_send, sel, sel_impl};

#[repr(C)]
#[cfg(target_os = "ios")]
struct UIEdgeInsets {
    top: f64,
    left: f64,
    bottom: f64,
    right: f64,
}

pub struct EdgeInsets {
    pub top: f64,
    pub left: f64,
    pub bottom: f64,
    pub right: f64,
}

impl fmt::Display for EdgeInsets {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "top: {}, bottom: {}, left: {}, right: {}",
            self.top, self.bottom, self.left, self.right
        )
    }
}

pub fn get_safe_area_insets() -> EdgeInsets {
    #[cfg(target_os = "ios")]
    unsafe {
        let ui_application: *mut Object = msg_send![class!(UIApplication), sharedApplication];
        let key_window: *mut Object = msg_send![ui_application, keyWindow];
        let safe_area_insets: UIEdgeInsets = msg_send![key_window, safeAreaInsets];
        let top = safe_area_insets.top as f64;
        let bottom = safe_area_insets.bottom as f64;
        let left = safe_area_insets.left as f64;
        let right = safe_area_insets.right as f64;

        EdgeInsets {
            top,
            bottom,
            left,
            right,
        }
    }
    #[cfg(not(target_os = "ios"))]
    EdgeInsets {
        top: 0.0,
        bottom: 0.0,
        left: 0.0,
        right: 0.0,
    }
}
