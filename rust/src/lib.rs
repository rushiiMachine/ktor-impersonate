pub(crate) mod jni_cache;
pub(crate) mod exception;
mod methods;

use jni::sys::{jint, JNI_ERR, JNI_VERSION_1_6};
use jni::JavaVM;
use std::ffi::c_void;
use std::sync::OnceLock;
use tokio::runtime::Runtime;

/// The runtime to be used for launching async tasks that need to use [JNIEnv].
pub(crate) static TOKIO_RUNTIME: OnceLock<Runtime> = OnceLock::new();

#[no_mangle]
pub extern "system" fn JNI_OnLoad(vm: JavaVM, _reserved: c_void) -> jint {
	let mut env = vm.get_env().unwrap();

	// Initialize logging backed per-platform
	#[cfg(target_os = "android")]
	android_log::init("KtorImpersonateNative").unwrap();

	// Initialize the JNI reference cache
	// SAFETY: init_cache does not create JNI local references
	if unsafe { !jni_cache::init_cache(env.unsafe_clone()) } {
		return JNI_ERR;
	}

	// SAFETY: from_raw always receives a valid pointer
	let vm_copy = unsafe { JavaVM::from_raw(vm.get_java_vm_pointer()) }.unwrap();

	// Initialize global tokio runtime
	let runtime = tokio::runtime::Builder::new_multi_thread()
		.on_thread_start(move || { vm_copy.attach_current_thread_as_daemon().expect("Failed to attach tokio thread to Java"); })
		.enable_time()
		.enable_io()
		.build();
	match runtime {
		Ok(rt) => TOKIO_RUNTIME.set(rt).unwrap(),
		Err(err) => throw!(env, &*format!("Failed to initialize tokio runtime: {err}"), JNI_ERR),
	}

	JNI_VERSION_1_6
}

// Note: This is never called on Android
#[no_mangle]
pub extern "system" fn JNI_OnUnload(_vm: JavaVM, _reserved: c_void) {
	jni_cache::release_cache();
}
