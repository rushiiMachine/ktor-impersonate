use bytes::Bytes;
use dashmap::DashMap;
use futures_core::stream::BoxStream;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, LazyLock, Mutex};
use tokio::task::AbortHandle;

/// Used for sequentially increasing IDs.
static NEXT_REQUEST_ID: AtomicU32 = AtomicU32::new(1);

/// Increments a global request ID counter.
/// The returned ID will never be `0`.
pub fn new_request_id() -> u32 {
	NEXT_REQUEST_ID.fetch_add(1, Ordering::Relaxed)
}

/// Request tasks that have not yet completed (including long-lived websockets)
/// This is in order to be able to cancel currently running requests.
pub static ACTIVE_REQUESTS: LazyLock<DashMap<u32, RequestTask>> =
	LazyLock::new(|| DashMap::with_capacity(20));

/// A currently executing request running in an async task.
/// This encapsulates a way to cancel it and retrieve data.
pub enum RequestTask {
	/// A single request awaiting a response (not SSE or Websocket).
	PendingResponse {
		/// A handle to the spawned task responsible for executing the request and returning the data through FFI.
		/// Once the response has been retrieved (excluding data), [abort] is set to [None] and [body] is populated.
		abort: Option<AbortHandle>,

		/// The response data stream that is populated once request has succeeded.
		/// This is used to stream chunks of the body across multiple calls.
		body: Option<Arc<Mutex<BoxStream<'static, Result<Bytes, rquest::Error>>>>>,
	},

	__NonExhaustive, // TODO: websockets
}
