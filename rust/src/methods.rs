use crate::{throw, throw_argument};
use catch_panic::catch_panic;
use dashmap::{DashMap, Entry};
use jni::objects::{JClass, JObject, JString};
use jni::signature::ReturnType;
use jni::sys::{jboolean, jint, jlong};
use jni::JNIEnv;
use jni_fn::jni_fn;
use log::debug;
use rand::Rng;
use rquest::tls::Impersonate;
use rquest::{Client, RequestBuilder};
use std::borrow::Cow;
use std::ops::Deref;
use std::str::FromStr;
use std::sync::LazyLock;
use tokio::task::AbortHandle;
use jnix::{FromJava, JnixEnv};

/// Request tasks that have not yet completed (including long-lived websockets)
/// This is in order to be able to cancel currently running requests.
static ACTIVE_REQUESTS: LazyLock<DashMap<i32, RequestTask>> = LazyLock::new(|| DashMap::with_capacity(20));

enum RequestTask {
	Single(AbortHandle),
	Continuous(()),
}

// ------------------------ JNI ------------------------ //

#[catch_panic]
#[jni_fn("dev.rushii.ktor_impersonate.Native")]
pub fn createClient<'l>(
	mut env: JNIEnv<'l>,
	_cls: JClass<'l>,
	config: JObject<'l>,
) -> jlong {
	let mut builder = Client::builder();

	let client_ptr = match builder.build() {
		Ok(client) => Box::leak(Box::new(client)) as *const Client,
		Err(err) => throw_argument!(env, &*format!("Failed to build rquest Client: {err}"))
	};

	client_ptr as jlong
}

#[catch_panic]
#[jni_fn("dev.rushii.ktor_impersonate.Native")]
pub fn destroyClient<'l>(
	_env: JNIEnv<'l>,
	_cls: JClass<'l>,
	client_ptr: jlong,
) {
	let client_ptr = client_ptr as *mut Client;
	if client_ptr.is_null() { return; }

	// Free the Box and decrease Client's Arc
	drop(unsafe { Box::from_raw(client_ptr) });
}

#[catch_panic]
#[jni_fn("dev.rushii.ktor_impersonate.Native")]
pub fn executeRequest<'l>(
	mut env: JNIEnv<'l>,
	_cls: JClass<'l>,
	client_ptr: jlong,
	callbacks: JObject<'l>,
	url: JString<'l>,
	http_method: JString<'l>,
	is_websocket: jboolean,
) -> jint {
	// Convert JNI types into rust types
	// SAFETY: Parameters are java/lang/String without a doubt
	let j_url = unsafe { env.get_string_unchecked(&url) }.unwrap();
	let j_http_method = unsafe { env.get_string_unchecked(&http_method) }.unwrap();
	let url: Cow<str> = j_url.deref().into();
	let http_method: Cow<str> = j_http_method.deref().into();

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
	if client_ptr == 0 { throw!(env, "Client is already closed!", -1); }
	let client = unsafe { &*(client_ptr as *mut Client) }.clone();

	// Create & setup request builder
	let builder = client.request(http_method, url);

	if is_websocket > 0 {
		execute_websocket(env, client, builder)
	} else {
		execute_request(env, client, builder)
	}
}

#[catch_panic]
#[jni_fn("dev.rushii.ktor_impersonate.Native")]
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

fn execute_request(mut env: JNIEnv, client: Client, builder: RequestBuilder) -> jint {
	let request = match builder.build() {
		Ok(req) => req,
		Err(err) => throw!(env, &*format!("Failed to build request: {err}"), -1),
	};

	let mut request_id: i32 = 0;
	let handle = tokio::spawn(async move {
		match client.execute(request).await {
			Err(err) => todo!(),
			Ok(resp) => {
				todo!()
			}
		};

		debug!("Clearing request {request_id}");
		ACTIVE_REQUESTS.remove(&request_id);
	});

	request_id = store_request_task(RequestTask::Single(handle.abort_handle()));
	request_id
}

fn execute_websocket(mut env: JNIEnv, client: Client, builder: RequestBuilder) -> jint {
	store_request_task(RequestTask::Continuous(()));
	todo!();
}
