use catch_panic::catch_panic;
use jni::objects::{GlobalRef, JMethodID};
use jni::strings::JNIString;
use jni::JNIEnv;

macro_rules! cache_ref {
    ($name:ident: $ty:ty) => { paste::paste! {
		#[allow(non_upper_case_globals)]
		static mut [<INNER_ $name>]: Option<$ty> = None;

		/// Unwraps this cache member as long as `JNI_OnUnload` has not been called yet.
		/// The values returned by this should be disposed of as quickly as possible and not held.
		#[allow(non_snake_case)]
		pub(crate) fn $name() -> $ty
			where $ty: Clone
		{
			unsafe { [<INNER_ $name>] .clone() }
				.expect("JNI cache already cleaned up")
		}
	}};
}

cache_ref!(ClassInvalidArgumentException: GlobalRef);
cache_ref!(ClassRuntimeException: GlobalRef);
cache_ref!(ClassNativeCallbacks: GlobalRef);
cache_ref!(MethodOnResponse: JMethodID);
cache_ref!(MethodOnError: JMethodID);

#[catch_panic(default = "false")]
pub(super) unsafe fn init_cache(mut env: JNIEnv) -> bool {
	INNER_ClassInvalidArgumentException = Some(make_class_ref(&mut env, "java/lang/InvalidArgumentException").unwrap());
	INNER_ClassRuntimeException = Some(make_class_ref(&mut env, "java/lang/RuntimeException").unwrap());
	INNER_ClassNativeCallbacks = Some(make_class_ref(&mut env, "dev/rushii/ktor_impersonate/Native$Callbacks").unwrap());
	INNER_MethodOnResponse = Some(env.get_method_id(&ClassNativeCallbacks(), "onResponse", "(ILjava/lang/String;)V").unwrap());
	INNER_MethodOnError = Some(env.get_method_id(&ClassNativeCallbacks(), "onError", "(Ljava/lang/String;)V").unwrap());
	true
}

/// Class refs should be deleted after all their member IDs have been.
/// Otherwise, if the class gets unloaded by the JVM, all the method/field IDs become invalid.s
#[catch_panic(default = "false")]
pub(super) unsafe fn release_cache(env: JNIEnv) -> bool {
	INNER_ClassInvalidArgumentException = None;
	INNER_ClassRuntimeException = None;
	INNER_MethodOnResponse = None;
	INNER_MethodOnError = None;
	INNER_ClassNativeCallbacks = None;
	true
}

fn make_class_ref<S: Into<JNIString>>(env: &mut JNIEnv, name: S) -> jni::errors::Result<GlobalRef> {
	env.find_class(name)
		.and_then(|cls| env.new_global_ref(cls))
}
