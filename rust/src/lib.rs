mod root_certs;
mod jni;

use std::sync::OnceLock;
use tokio::runtime::Runtime;

/// The runtime to be used for launching async tasks that need to use [JNIEnv].
pub(crate) static TOKIO_RUNTIME: OnceLock<Runtime> = OnceLock::new();
