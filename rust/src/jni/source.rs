use crate::jni::cache;
use crate::requests::{RequestTask, ACTIVE_REQUESTS};
use crate::{throw, throw_argument, TOKIO_RUNTIME};
use catch_panic::catch_panic;
use futures_util::StreamExt;
use jni::errors::Error as JNIError;
use jni::objects::{JObject, JValue};
use jni::signature::{Primitive, ReturnType};
use jni::sys::{jint, jlong};
use jni::JNIEnv;
use jni_fn::jni_fn;

#[catch_panic]
#[jni_fn("dev.rushii.ktor_impersonate.internal.ResponseSource")]
pub fn init<'l>(mut env: JNIEnv<'l>, instance: JObject<'l>) {
	// Get the request id stored in a field in the class
	let request_id = match get_request_id(&mut env, &instance) {
		Err(err) => throw!(env, &*format!("Failed to get request id: {err:?}")),
		Ok(None) => throw_argument!(env, "Target request id does not exist"),
		Ok(Some(id)) => id,
	};

	if let Some(entry) = ACTIVE_REQUESTS.get(&request_id) {
		match entry.value() {
			RequestTask::PendingResponse { body, .. } => {
				if body.is_none() {
					throw_argument!(env, "Target request id does not have a body stream")
				}
			}
			_ => throw_argument!(env, "Target request id is of wrong task type"),
		}
	}
}

#[catch_panic]
#[jni_fn("dev.rushii.ktor_impersonate.internal.ResponseSource")]
pub fn close<'l>(mut env: JNIEnv<'l>, instance: JObject<'l>) {
	// Get the request id stored in a field in the class
	let request_id = match get_request_id(&mut env, &instance) {
		Err(err) => throw!(env, &*format!("Failed to get request id: {err:?}")),
		Ok(None) => return,
		Ok(Some(id)) => id,
	};

	if let Err(err) = clear_request(&mut env, &instance, request_id) {
		throw!(env, &*format!("Failed to clear request id: {err:?}"));
	}
}

#[catch_panic]
#[jni_fn("dev.rushii.ktor_impersonate.internal.ResponseSource")]
pub fn readAtMostTo<'l>(
	mut env: JNIEnv<'l>,
	instance: JObject<'l>,
	sink: JObject<'l>,
	min_bytes: jlong,
) -> jlong {
	// Get the request id stored in a field in the class
	let request_id = match get_request_id(&mut env, &instance) {
		Err(err) => throw!(env, &*format!("Failed to get request id: {err:?}"), 0),
		Ok(None) => return -1,
		Ok(Some(id)) => id,
	};

	// Get the body stream from the global ACTIVE_REQUESTS store
	let stream_mutex = match ACTIVE_REQUESTS.get(&request_id) {
		None => return -1,
		Some(entry) => {
			let body_cell = match entry.value() {
				RequestTask::PendingResponse { body, .. } => body,
				_ => unreachable!(),
			};
			match body_cell.as_ref() {
				None => panic!("BUG: response stream should have been initialized by the time the ResponseSource is read"),
				Some(mutex_arc) => mutex_arc.clone(),
			}
		}
	};
	let mut stream = stream_mutex
		.lock()
		.unwrap(); // Propagate poison error

	// Get tokio runtime to run async stream collector
	let runtime_lock = TOKIO_RUNTIME.read().expect("runtime lock poisoned");
	let runtime = runtime_lock.as_ref().expect("runtime not initialized");

	let mut total_bytes: usize = 0;
	while total_bytes < min_bytes as usize {
		// FIXME: This should not block! Figure out a way to use a suspend RawSource to connect this to a cancellableCoroutine instead
		let result = runtime.block_on(async {
			stream.next().await
		});
		let bytes = match result {
			// EOF
			None => {
				if let Err(err) = clear_request(&mut env, &instance, request_id) {
					throw!(env, &*format!("Failed to clear request id: {err:?}"), 0);
				}
				break;
			}
			Some(result) => result.unwrap(), // TODO: handle errors
		};

		// Convert the bytes chunk into a JVM bytearray
		let bytes_len = bytes.len();
		let bytes_obj = match env.byte_array_from_slice(bytes.as_ref()) {
			Ok(arr) => env.auto_local(arr),
			Err(err) => throw!(env, &*format!("Failed to make response chunk bytearray: {err:?}"), 0)
		};

		total_bytes += bytes_len;
		drop(bytes);

		let result = unsafe {
			env.call_method_unchecked(
				&sink,
				cache::Sink_write(),
				ReturnType::Primitive(Primitive::Void),
				&[
					JValue::from(&*bytes_obj).as_jni(),
					JValue::from(0).as_jni(),
					JValue::from(bytes_len as jint).as_jni()
				],
			)
		};
		if let Err(err) = result {
			throw!(env, &*format!("Failed to write response chunk to sink: {err:?}"), 0);
		}
	}

	if total_bytes == 0 { -1 } else { total_bytes as jlong }
}

/// Extract the request ID from the `ResponseEngine#requestId` field.
fn get_request_id(env: &mut JNIEnv, source_obj: &JObject) -> Result<Option<u32>, JNIError> {
	let id = env.get_field_unchecked(
		source_obj,
		cache::ResponseSource_requestId(),
		ReturnType::Primitive(Primitive::Int),
	)?.i()? as u32;

	match id {
		0 => Ok(None),
		id => Ok(Some(id)),
	}
}

/// Clears the request ID from the `ResponseEngine#requestId` field,
/// and removes the request from [ACTIVE_REQUESTS].
fn clear_request(env: &mut JNIEnv, source_obj: &JObject, request_id: u32) -> Result<(), JNIError> {
	ACTIVE_REQUESTS.remove(&request_id);
	env.set_field_unchecked(&source_obj, cache::ResponseSource_requestId(), JValue::from(0))?;
	Ok(())
}
