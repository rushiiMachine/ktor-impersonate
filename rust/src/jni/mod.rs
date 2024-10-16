use crate::{throw, TOKIO_RUNTIME};
use jni::sys::{jint, JNI_ERR, JNI_VERSION_1_6};
use jni::JavaVM;
use std::ffi::c_void;

mod exception;
mod cache;
mod methods;
mod headers;
mod config;
mod utils;

#[no_mangle]
pub extern "system" fn JNI_OnLoad(vm: JavaVM, _reserved: c_void) -> jint {
	let mut env = vm.get_env().unwrap();

	// Initialize logging backend for each platform
	#[cfg(target_os = "android")]
	android_log::init("KtorImpersonateNative").unwrap();

	// Initialize the JNI reference cache
	// SAFETY: init_cache does not create JNI local references
	if unsafe { !cache::init_cache(env.unsafe_clone()) } {
		return JNI_ERR;
	}

	// SAFETY: from_raw always receives a valid pointer
	let vm_copy = unsafe { JavaVM::from_raw(vm.get_java_vm_pointer()) }.unwrap();

	// Initialize global tokio runtime
	let mut runtime_mut = TOKIO_RUNTIME.write().expect("runtime lock poisoned");
	let runtime = tokio::runtime::Builder::new_multi_thread()
		.on_thread_start(move || { vm_copy.attach_current_thread_as_daemon().expect("failed to attach worker thread to JVM"); })
		.enable_time()
		.enable_io()
		.build();
	match runtime {
		Err(err) => throw!(env, &*format!("Failed to initialize tokio runtime: {err}"), JNI_ERR),
		Ok(rt) => {
			*runtime_mut = Some(rt);
			drop(runtime_mut);
		}
	};

	JNI_VERSION_1_6
}

// Note: This is never called on Android
#[no_mangle]
pub extern "system" fn JNI_OnUnload(_vm: JavaVM, _reserved: c_void) {
	cache::release_cache();

	// Shutdown the tokio runtime
	drop(TOKIO_RUNTIME.write().expect("runtime lock poisoned").take());
}
