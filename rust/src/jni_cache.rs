use catch_panic::catch_panic;
use jni::objects::GlobalRef;
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
				.expect("JNI cache member already cleaned up during JNI_OnUnload")
		}
	}};
}

cache_ref!(ClassInvalidArgumentException: GlobalRef);
cache_ref!(ClassRuntimeException: GlobalRef);

#[catch_panic(default = "false")]
pub(super) unsafe fn init_cache(mut env: JNIEnv) -> bool {
	INNER_ClassInvalidArgumentException = Some(make_class_ref(&mut env, "java/lang/InvalidArgumentException").unwrap());
	INNER_ClassRuntimeException = Some(make_class_ref(&mut env, "java/lang/RuntimeException").unwrap());
	true
}

/// Class refs should be deleted after all their member IDs have been.
/// Otherwise, if the class gets unloaded by the JVM, all the method/field IDs become invalid.s
#[catch_panic(default = "false")]
pub(super) unsafe fn release_cache(env: JNIEnv) -> bool {
	INNER_ClassInvalidArgumentException = None;
	INNER_ClassRuntimeException = None;
	true
}

fn make_class_ref<S: Into<JNIString>>(env: &mut JNIEnv, name: S) -> jni::errors::Result<GlobalRef> {
	env.find_class(name)
		.and_then(|cls| env.new_global_ref(cls))
}
