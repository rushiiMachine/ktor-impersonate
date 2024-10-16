use crate::jni::cache;
use crate::jni::utils::boxed_jni_to_primitive;
use jni::errors::Error as JNIError;
use jni::objects::JObject;
use jni::signature::{Primitive, ReturnType};
use jni::JNIEnv;
use rquest::tls::Impersonate;
use rquest::ClientBuilder;
use std::str::FromStr;
use std::time::Duration;

/// Applies the JVM-side impersonate config to a rquest [ClientBuilder].
/// [config_obj]: An instance of `dev/rushii/ktor_impersonate/ImpersonateConfig`
pub fn apply_jni_config(
	env: &mut JNIEnv,
	config_obj: &JObject,
	mut client: ClientBuilder,
) -> Result<ClientBuilder, JNIError> {
	let config = env.with_local_frame(0, |env| unsafe {
		get_jni_config(env, config_obj)
	})?;

	client = client
		.connection_verbose(config.verbose_logging);
	if let Some(preset) = config.preset.as_deref() {
		client = client.impersonate(Impersonate::from_str(preset)
			.expect("BUG: invalid impersonate preset"));
	}
	if let Some(duration) = config.request_timeout {
		client = client.timeout(duration);
	}
	if let Some(duration) = config.connect_timeout {
		client = client.connect_timeout(duration);
	}
	if let Some(duration) = config.idle_timeout {
		client = client.pool_idle_timeout(duration);
	}
	if let Some(enabled) = config.invalid_certs {
		client = client.danger_accept_invalid_certs(enabled);
	}
	if let Some(enabled) = config.https_only {
		client = client.https_only(enabled);
	}

	Ok(client)
}

unsafe fn get_jni_config(env: &mut JNIEnv, config_obj: &JObject) -> Result<ImpersonateConfig, JNIError> {
	if !env.is_instance_of(config_obj, &cache::ImpersonateConfig())? {
		panic!("supplied config_obj is not of subtype ImpersonateConfig")
	}

	let verbose_logging = env.call_method_unchecked(config_obj, cache::ImpersonateConfig_getVerboseLogging(), ReturnType::Primitive(Primitive::Boolean), &[])?.z()?;

	let preset = env.call_method_unchecked(config_obj, cache::ImpersonateConfig_getPreset(), ReturnType::Object, &[])?.l()?;
	let preset = if preset.is_null() { None } else {
		Some(env.get_string((&preset).into())?)
	};

	let request_timeout = env.call_method_unchecked(config_obj, cache::ImpersonateConfig_getRequestTimeoutMillis(), ReturnType::Object, &[])?.l()?;
	let request_timeout = boxed_jni_to_primitive(env, &request_timeout)?.map(|v| v.j().unwrap());

	let connect_timeout = env.call_method_unchecked(config_obj, cache::ImpersonateConfig_getConnectTimeoutMillis(), ReturnType::Object, &[])?.l()?;
	let connect_timeout = boxed_jni_to_primitive(env, &connect_timeout)?.map(|v| v.j().unwrap());

	let idle_timeout = env.call_method_unchecked(config_obj, cache::ImpersonateConfig_getIdleTimeout(), ReturnType::Object, &[])?.l()?;
	let idle_timeout = boxed_jni_to_primitive(env, &idle_timeout)?.map(|v| v.j().unwrap());

	let invalid_certs = env.call_method_unchecked(config_obj, cache::ImpersonateConfig_getAllowInvalidCertificates(), ReturnType::Object, &[])?.l()?;
	let invalid_certs = boxed_jni_to_primitive(env, &invalid_certs)?.map(|v| v.z().unwrap());

	let https_only = env.call_method_unchecked(config_obj, cache::ImpersonateConfig_getHttpsOnly(), ReturnType::Object, &[])?.l()?;
	let https_only = boxed_jni_to_primitive(env, &https_only)?.map(|v| v.z().unwrap());

	Ok(ImpersonateConfig {
		verbose_logging,
		preset: preset.map(|str| str.into()),
		request_timeout: request_timeout.map(|millis| Duration::from_millis(millis as u64)),
		connect_timeout: connect_timeout.map(|millis| Duration::from_millis(millis as u64)),
		idle_timeout: idle_timeout.map(|millis| Duration::from_millis(millis as u64)),
		invalid_certs,
		https_only,
	})
}

#[derive(Debug)]
struct ImpersonateConfig {
	verbose_logging: bool,
	preset: Option<String>,
	request_timeout: Option<Duration>,
	connect_timeout: Option<Duration>,
	idle_timeout: Option<Duration>,
	invalid_certs: Option<bool>,
	https_only: Option<bool>,
}
