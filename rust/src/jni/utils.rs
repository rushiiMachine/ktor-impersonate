use crate::jni::cache;
use crate::throw_argument;
use jni::errors::Error as JNIError;
use jni::objects::{JObject, JObjectArray, JString, JValueOwned};
use jni::signature::{Primitive, ReturnType};
use jni::JNIEnv;
use std::borrow::Cow;

/// Unwraps all the boxed primitive objects into their inner primitive value.
/// If [boxed_obj] is null, [None] is returned.
pub fn boxed_jni_to_primitive<'local>(
	env: &mut JNIEnv<'local>,
	boxed_obj: &JObject,
) -> Result<Option<JValueOwned<'local>>, JNIError> {
	if boxed_obj.is_null() { return Ok(None); }

	let class = env.auto_local(env.get_object_class(boxed_obj)?);
	let class_name_obj = unsafe { env.call_method_unchecked(&*class, cache::Class_getName(), ReturnType::Object, &[]) }?.l()?;
	let class_name_obj = env.auto_local(JString::from(class_name_obj));
	let class_name_str = unsafe { env.get_string_unchecked(&*class_name_obj) }?;
	let class_name: Cow<str> = (*class_name_str).into();

	let (method, primitive) = match class_name.as_ref() {
		"java.lang.Boolean" => (cache::Boolean_booleanValue(), Primitive::Boolean),
		"java.lang.Byte" |
		"java.lang.Character" |
		"java.lang.Double" |
		"java.lang.Float" |
		"java.lang.Integer" => unimplemented!("other primitives"),
		"java.lang.Long" => (cache::Long_longValue(), Primitive::Long),
		"java.lang.Short" => unimplemented!("other primitives"),
		_ => throw_argument!(env, "boxed_value is not a boxed primitive", Err(JNIError::JavaException)),
	};
	let value = unsafe { env.call_method_unchecked(boxed_obj, method, ReturnType::Primitive(primitive), &[]) }?;
	Ok(Some(value))
}

/// Gets all the values contained within a List<String>
/// [list_obj]: Must be an instance of `java/util/List` with generic type T as `java/lang/String`.
pub unsafe fn get_string_list_values(env: &mut JNIEnv, list_obj: &JObject) -> Result<Vec<String>, JNIError> {
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
