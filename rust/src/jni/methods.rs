use crate::jni::headers::{headers_to_jni, jni_to_headers};
use crate::jni::{cache, config};
use crate::{root_certs, throw, throw_argument, TOKIO_RUNTIME};
use arraystring::typenum::U8;
use arraystring::ArrayString;
use catch_panic::catch_panic;
use dashmap::{DashMap, Entry};
use jni::objects::{GlobalRef, JClass, JObject, JString, JValueGen, JValueOwned};
use jni::signature::{Primitive, ReturnType};
use jni::sys::{jboolean, jint, jlong};
use jni::{JNIEnv, JavaVM};
use jni_fn::jni_fn;
use log::debug;
use rand::Rng;
use rquest::{Client, RequestBuilder, Response};
use std::borrow::Cow;
use std::fmt::Write;
use std::ops::Deref;
use std::str::FromStr;
use std::sync::LazyLock;
use tokio::task::AbortHandle;

/// Request tasks that have not yet completed (including long-lived websockets)
/// This is in order to be able to cancel currently running requests.
static ACTIVE_REQUESTS: LazyLock<DashMap<i32, RequestTask>> = LazyLock::new(|| DashMap::with_capacity(20));

enum RequestTask {
	Single(AbortHandle),
	Continuous(()),
}

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
		execute_websocket(env, client, builder)
	} else {
		execute_request(env, callbacks, client, builder)
	}
}

#[catch_panic]
#[jni_fn("dev.rushii.ktor_impersonate.internal.NativeEngine")]
pub fn cancelRequest<'l>(
	_env: JNIEnv<'l>,
	_cls: JClass<'l>,
	request_id: jint,
) {
	debug!("Cancelling request {request_id}");
	match ACTIVE_REQUESTS.remove(&request_id) {
		None => return,
		Some((_, task)) => match task {
			RequestTask::Single(handle) => handle.abort(),
			RequestTask::Continuous(_) => todo!(),
		}
	}
}

// ------------------------ JNI Callbacks ------------------------ //

fn callback_response(vm: JavaVM, callbacks: GlobalRef, response: Response) {
	// We assume this thread is already attached to the VM based on the tokio runtime config
	let mut env = vm.get_env().expect("Thread is not attached to JavaVM");

	let status = response.status().as_u16();
	let status_jni = JValueOwned::from(status as i32).as_jni();

	// Format HTTP version to string
	let mut version = ArrayString::<U8>::new();
	write!(version, "{:?}", response.version())
		.expect("Unexpected HTTP version");
	let version_jni = JValueOwned::from(env.new_string(version).unwrap()).as_jni();

	// Convert the headers to jni
	let headers_jni = headers_to_jni(&mut env, response.headers())
		.map(JValueOwned::from)
		.expect("failed to convert headers map") // TODO: return error like callback_request_error does
		.as_jni();

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

fn callback_request_error(vm: JavaVM, callbacks: GlobalRef, error: rquest::Error) {
	// We assume this thread is already attached to the VM based on the tokio runtime config
	let mut env = vm.get_env().expect("Thread is not attached to JavaVM");

	let message = format!("Failed to execute request: {error}");
	let message_jni = JValueGen::from(env.new_string(message).unwrap()).as_jni();

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

/// Generate a random id for this request and store a cancellation handle into ACTIVE_REQUESTS for later
fn store_request_task(task: RequestTask) -> i32 {
	loop {
		let id = rand::thread_rng().gen_range(1..=i32::MAX);
		match ACTIVE_REQUESTS.entry(id) {
			Entry::Occupied(_) => { continue; }
			Entry::Vacant(entry) => {
				entry.insert(task);
				break id;
			}
		}
	}
}

fn execute_request(mut env: JNIEnv, callbacks: GlobalRef, client: Client, builder: RequestBuilder) -> jint {
	let mut request_id: i32 = 0;

	let request = match builder.build() {
		Ok(req) => req,
		Err(err) => throw!(env, &*format!("Failed to build request: {err}"), -1),
	};

	let runtime_lock = TOKIO_RUNTIME.read().expect("runtime lock poisoned");
	let runtime = runtime_lock.as_ref().expect("runtime not initialized");

	let vm = env.get_java_vm().unwrap();
	let handle = runtime.spawn(async move {
		match client.execute(request).await {
			Err(err) => callback_request_error(vm, callbacks, err),
			Ok(resp) => callback_response(vm, callbacks, resp),
		};

		debug!("Clearing request {request_id}");
		ACTIVE_REQUESTS.remove(&request_id);
	});

	request_id = store_request_task(RequestTask::Single(handle.abort_handle()));
	request_id
}

fn execute_websocket(_env: JNIEnv, _client: Client, _builder: RequestBuilder) -> jint {
	store_request_task(RequestTask::Continuous(()));
	todo!();
}
