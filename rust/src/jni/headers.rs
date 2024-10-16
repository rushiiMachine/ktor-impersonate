use crate::jni::cache;
use crate::throw_argument;
use jni::errors::Error as JNIError;
use jni::objects::{JObject, JObjectArray, JString, JValue, JValueGen, JValueOwned};
use jni::signature::{Primitive, ReturnType};
use jni::JNIEnv;
use rquest::header::{HeaderMap, HeaderName, HeaderValue};
use std::borrow::Cow;
use std::str::FromStr;

/// Converts a rquest [HeaderMap] into a JVM Ktor `io/ktor/http/Headers` instance.
pub fn headers_to_jni<'local>(env: &mut JNIEnv<'local>, headers: &HeaderMap) -> Result<JObject<'local>, JNIError> {
	let key_count_jni = JValueOwned::from(headers.keys_len() as i32).as_jni();

	// SAFETY: arguments are correct type
	let builder_obj = unsafe {
		env.new_object_unchecked(
			&cache::HeadersBuilder(),
			cache::HeadersBuilder_init(),
			&[key_count_jni],
		)?
	};

	for (key, value) in headers {
		env.with_local_frame(2, |env| {
			let value_str = match value.to_str() {
				Ok(str) => Cow::from(str),
				Err(_) => String::from_utf8_lossy(value.as_bytes())
			};

			let key_jni = JValueGen::from(env.new_string(key)?).as_jni();
			let value_jni = JValueGen::from(env.new_string(&*value_str)?).as_jni();

			// SAFETY: obj is not null and arguments are correct
			unsafe {
				env.call_method_unchecked(
					&builder_obj,
					cache::StringValuesBuilder_append(),
					ReturnType::Primitive(Primitive::Void),
					&[key_jni, value_jni],
				)
			}?;

			// Explicitly provide the Error type otherwise inferring fails
			Result::<_, JNIError>::Ok(())
		})?;
	}

	// SAFETY: arguments are correct type
	let headers_obj = unsafe {
		env.call_method_unchecked(
			&builder_obj,
			cache::HeadersBuilder_build(),
			ReturnType::Object,
			&[],
		)
	}?.l()?;

	Ok(headers_obj)
}

/// Converts a JVM Ktor `io/ktor/http/Headers` instance into a rquest [HeaderMap].
pub fn jni_to_headers(env: &mut JNIEnv, headers_obj: &JObject) -> Result<HeaderMap, JNIError> {
	if !env.is_instance_of(headers_obj, &cache::StringValues())? {
		panic!("supplied headers_obj is not of subtype StringValues")
	}

	// Obtain all the header names (Headers -> Set -> Array -> rust Vec<String>)
	let keys_array = unsafe { get_stringvalues_keys(env, headers_obj) }?;
	let keys_array = env.auto_local(keys_array);
	let keys_count = env.get_array_length(&*keys_array)?;

	let mut headers = HeaderMap::new();

	for key_i in 0..keys_count {
		let key = env.get_object_array_element(&*keys_array, key_i)?;
		let key = env.auto_local(JString::from(key));
		let key_string = unsafe { env.get_string_unchecked(&*key)? };
		let key_string: Cow<str> = (*key_string).into();

		let values_list = unsafe {
			env.call_method_unchecked(
				headers_obj,
				cache::StringValues_getAll(),
				ReturnType::Object,
				&[JValue::from(&*key).as_jni()],
			)
		}?.l()?;
		let values_list = env.auto_local(values_list);
		let values = unsafe { get_string_list_values(env, &*values_list) }?;

		let header_name = match HeaderName::from_str(&*key_string) {
			Ok(v) => v,
			Err(_) => throw_argument!(env, "Invalid header name!", Err(JNIError::JavaException)),
		};

		headers.reserve(values.len());
		for value in values {
			let header_value = match HeaderValue::try_from(&value) {
				Ok(v) => v,
				Err(_) => throw_argument!(env, "Invalid header value", Err(JNIError::JavaException)),
			};

			headers.append(header_name.clone(), header_value);
		}
	}

	Ok(headers)
}

/// Gets all the keys into an object array of type `String[]`.
/// [headers_obj]: Must be an instance of `io/ktor/util/StringValues`.
unsafe fn get_stringvalues_keys<'local>(
	env: &mut JNIEnv<'local>,
	headers_obj: &JObject,
) -> Result<JObjectArray<'local>, JNIError> {
	let keys_set = env.call_method_unchecked(headers_obj, cache::StringValues_names(), ReturnType::Object, &[])?.l()?;
	let keys_set = env.auto_local(keys_set);

	let keys_array_obj = env.call_method_unchecked(&keys_set, cache::Set_toArray(), ReturnType::Array, &[])?.l()?;
	Ok(JObjectArray::from(keys_array_obj))
}

/// Gets all the values contained within a List<String>
/// [list_obj]: Must be an instance of `java/util/List` with generic type T as `java/lang/String`.
unsafe fn get_string_list_values(env: &mut JNIEnv, list_obj: &JObject) -> Result<Vec<String>, JNIError> {
	let array = env.call_method_unchecked(list_obj, cache::List_toArray(), ReturnType::Array, &[])?.l()?;
	let array = env.auto_local(JObjectArray::from(array));
	let array_length = env.get_array_length(&*array)?;

	let mut vec: Vec<String> = Vec::with_capacity(array_length as usize);

	for i in 0..array_length {
		let item = env.get_object_array_element(&*array, i)?;
		let item = env.auto_local(JString::from(item));

		let string = env.get_string_unchecked(&*item)?;
		vec.push(string.into());
	}
	Ok(vec)
}
