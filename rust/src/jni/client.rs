use crate::jni::headers::{headers_to_jni, jni_to_headers};
use crate::jni::{cache, config};
use crate::requests::{new_request_id, RequestTask, ACTIVE_REQUESTS};
use crate::{root_certs, throw, throw_argument, TOKIO_RUNTIME};
use catch_panic::catch_panic;
use dashmap::Entry;
use futures_util::StreamExt;
use jni::objects::{GlobalRef, JClass, JObject, JString, JValueGen, JValueOwned};
use jni::signature::{Primitive, ReturnType};
use jni::sys::{jboolean, jint, jlong};
use jni::{JNIEnv, JavaVM};
use jni_fn::jni_fn;
use rquest::{Client, Request, Response};
use std::borrow::Cow;
use std::fmt::Write;
use std::ops::Deref;
use std::str::FromStr;
use std::sync::{Arc, Mutex};

// ------------------------ JNI ------------------------ //

#[catch_panic]
#[jni_fn("dev.rushii.ktor_impersonate.internal.NativeEngine")]
pub fn createClient<'l>(
	mut env: JNIEnv<'l>,
	_cls: JClass<'l>,
	config: JObject<'l>,
) -> jlong {
	let mut builder = Client::builder();

	builder = config::apply_jni_config(&mut env, &config, builder)
		.expect("failed to apply config");

	match root_certs::get_cached_verify_store() {
		Ok(store) => builder = builder.ca_cert_store(store),
		Err(err) => throw!(env, &*format!("Failed to load certificates: {err:#?}"), 0),
	}

	let client_ptr = match builder.build() {
		Ok(client) => Box::leak(Box::new(client)) as *const Client,
		Err(err) => throw_argument!(env, &*format!("Failed to build rquest Client: {err}"), 0)
	};

	client_ptr as jlong
}

#[catch_panic]
#[jni_fn("dev.rushii.ktor_impersonate.internal.NativeEngine")]
pub fn destroyClient<'l>(
	_env: JNIEnv<'l>,
	_cls: JClass<'l>,
	client_ptr: jlong,
) {
	let client_ptr = client_ptr as *mut Client;
	if client_ptr.is_null() { return; }

	// Free the Box and decrease Client's Arc count
	// SAFETY: This works as long as the Java-side invariant is preserved
	drop(unsafe { Box::from_raw(client_ptr) });
}

#[catch_panic]
#[jni_fn("dev.rushii.ktor_impersonate.internal.NativeEngine")]
pub fn executeRequest<'l>(
	mut env: JNIEnv<'l>,
	_cls: JClass<'l>,
	client_ptr: jlong,
	callbacks: JObject<'l>,
	url: JString<'l>,
	http_method: JString<'l>,
	headers: JObject<'l>,
	is_websocket: jboolean,
) -> jint {
	// Convert JNI types into rust types
	// SAFETY: Parameters are java/lang/String without a doubt
	let j_url = unsafe { env.get_string_unchecked(&url) }.unwrap();
	let j_http_method = unsafe { env.get_string_unchecked(&http_method) }.unwrap();
	let url: Cow<str> = j_url.deref().into();
	let http_method: Cow<str> = j_http_method.deref().into();
	let headers = jni_to_headers(&mut env, &headers).expect("failed to get headers from jni");
	let callbacks = env.new_global_ref(callbacks).unwrap();

	// Parse url & http method
	let url = match rquest::Url::parse(url.as_ref()) {
		Err(err) => throw_argument!(env, &*format!("Failed to parse url: {err}"), -1),
		Ok(url) => url,
	};
	let http_method = match rquest::Method::from_str(http_method.as_ref()) {
		Err(_) => throw_argument!(env, "HTTP method cannot be of 0 length", -1),
		Ok(mtd) => mtd,
	};

	// Retrieve rquest::Client from a pointer stored in the class
	// SAFETY: This works as long as the Java-side invariant is preserved
	if client_ptr == 0 { throw!(env, "Client is already closed!", -1); }
	let client = unsafe { &*(client_ptr as *mut Client) }.clone();

	// Create & setup request builder
	let builder = client.request(http_method, url)
		.headers(headers);

	if is_websocket > 0 {
		todo!()
	} else {
		let request = match builder.build() {
			Ok(req) => req,
			Err(err) => throw!(env, &*format!("Failed to build request: {err}"), -1),
		};

		execute_request(env, callbacks, client, request)
	}
}

#[catch_panic]
#[jni_fn("dev.rushii.ktor_impersonate.internal.NativeEngine")]
pub fn cancelRequest<'l>(
	_env: JNIEnv<'l>,
	_cls: JClass<'l>,
	request_id: u32, // matches jint with different representation
) {
	match ACTIVE_REQUESTS.remove(&request_id).map(|kv| kv.1) {
		Some(RequestTask::PendingResponse { abort, .. }) if abort.is_some() => {
			abort.unwrap().abort();
		}
		_ => return,
	}
}

// ------------------------ JNI Callbacks ------------------------ //

fn callback_response(vm: JavaVM, callbacks: GlobalRef, request_id: u32, response: Response) {
	// We assume this thread is already attached to the VM based on the tokio runtime config
	let mut env = vm.get_env().expect("Thread is not attached to JavaVM");

	let status = response.status().as_u16();
	let status_jni = JValueOwned::from(status as i32).as_jni();

	// Format HTTP version to string
	let mut version = String::with_capacity(8);
	write!(version, "{:?}", response.version()).unwrap();
	let version_jni = JValueOwned::from(env.new_string(version).unwrap()).as_jni();

	// Convert the headers to jni
	let headers_jni = headers_to_jni(&mut env, response.headers())
		.map(JValueOwned::from)
		.expect("failed to convert headers map") // TODO: return error like callback_request_error does
		.as_jni();

	// Store the response body into the global ACTIVE_REQUESTS and remove the AbortHandle (task is almost finished)
	if let Some(mut entry) = ACTIVE_REQUESTS.get_mut(&request_id) {
		match entry.value_mut() {
			RequestTask::PendingResponse { abort, body } => {
				let stream = response.bytes_stream().boxed();
				*abort = None;
				*body = Some(Arc::new(Mutex::new(stream)));
			}
			_ => unreachable!(),
		}
	} else {
		// This request has already been cancelled
	}

	// SAFETY: Method ID is always valid and sig types are correct
	unsafe {
		env.call_method_unchecked(
			callbacks,
			&cache::NativeCallbacks_onResponse(),
			ReturnType::Primitive(Primitive::Void),
			&[version_jni, status_jni, headers_jni],
		).expect("Failed to invoke onResponse callback");
	};
}

fn callback_request_error(vm: JavaVM, callbacks: GlobalRef, request_id: u32, error: rquest::Error) {
	// We assume this thread is already attached to the VM based on the tokio runtime config
	let mut env = vm.get_env().expect("Thread is not attached to JavaVM");

	let message = format!("Failed to execute request: {error}");
	let message_jni = JValueGen::from(env.new_string(message).unwrap()).as_jni();

	// Remove the request record from ACTIVE_REQUESTS
	ACTIVE_REQUESTS.remove(&request_id);

	// SAFETY: Method ID is always valid and sig types are correct
	unsafe {
		env.call_method_unchecked(
			callbacks,
			&cache::NativeCallbacks_onError(),
			ReturnType::Primitive(Primitive::Void),
			&[message_jni],
		).expect("Failed to invoke onError callback");
	}
}

// ------------------------ Other ------------------------ //

fn execute_request(env: JNIEnv, callbacks: GlobalRef, client: Client, request: Request) -> jint {
	let runtime_lock = TOKIO_RUNTIME.read().expect("runtime lock poisoned");
	let runtime = runtime_lock.as_ref().expect("runtime not initialized");

	let request_id = new_request_id();
	let vm = env.get_java_vm().unwrap();
	let task_handle = runtime.spawn(async move {
		let result = client.execute(request).await;

		match result {
			Err(err) => callback_request_error(vm, callbacks, request_id, err),
			Ok(resp) => callback_response(vm, callbacks, request_id, resp),
		};
	});

	match ACTIVE_REQUESTS.entry(request_id) {
		Entry::Occupied(_) => panic!("BUG: broken atomic or id overflow"),
		Entry::Vacant(entry) => entry.insert(RequestTask::PendingResponse {
			abort: Some(task_handle.abort_handle()),
			body: None,
		}),
	};

	request_id as jint
}
