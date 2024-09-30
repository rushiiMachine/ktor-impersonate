use jni::sys::{jint, JNI_ERR, JNI_VERSION_1_6};
use jni::JavaVM;
use log::info;
use std::ffi::c_void;

#[no_mangle]
pub extern "system" fn JNI_OnLoad(vm: JavaVM, _reserved: c_void) -> jint {
    #[cfg(target_os = "android")]
    android_log::init("KtorImpersonateNative").unwrap();

    info!("test");

    JNI_VERSION_1_6
}

// Note: This is never called on Android
#[no_mangle]
pub extern "system" fn JNI_OnUnload(_vm: JavaVM, _reserved: c_void) {}
