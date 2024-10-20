mod root_certs;
mod jni;
mod requests;

use std::sync::RwLock;
use tokio::runtime::Runtime;

/// The runtime to be used for launching async tasks that need to use [JNIEnv].
pub(crate) static TOKIO_RUNTIME: RwLock<Option<Runtime>> = RwLock::new(None);

/// Initialize a logging backend for each platform.
pub(crate) fn init_logging() {
	#[cfg(target_os = "android")]
	android_log::init("KtorImpersonateNative").unwrap();

	#[cfg(any(target_os = "ios", target_os = "macos"))]
	oslog::OsLogger::new("dev.rushii.ktor_impersonate")
		.init()
		.unwrap();

	#[cfg(not(any(target_os = "android", target_os = "ios")))]
	env_logger::init();

	log_panics::init();
}
