mod root_certs;
mod jni;

use std::sync::RwLock;
use tokio::runtime::Runtime;

/// The runtime to be used for launching async tasks that need to use [JNIEnv].
pub(crate) static TOKIO_RUNTIME: RwLock<Option<Runtime>> = RwLock::new(None);
