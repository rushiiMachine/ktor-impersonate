pub(crate) mod jni_cache;

use jni::sys::{jint, JNI_ERR, JNI_VERSION_1_6};
use jni::JavaVM;
use std::ffi::c_void;

#[no_mangle]
pub extern "system" fn JNI_OnLoad(vm: JavaVM, _reserved: c_void) -> jint {
    let env = vm.get_env().unwrap();

    #[cfg(target_os = "android")]
    android_log::init("KtorImpersonateNative").unwrap();

    if unsafe { !jni_cache::init_cache(env) } {
        return JNI_ERR;
    }

    JNI_VERSION_1_6
}

// Note: This is never called on Android
#[no_mangle]
pub extern "system" fn JNI_OnUnload(vm: JavaVM, _reserved: c_void) {
    let env = vm.get_env().unwrap();

    unsafe { jni_cache::release_cache(env); }
}
