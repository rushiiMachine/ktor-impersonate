use catch_panic::catch_panic;
use jni::objects::{GlobalRef, JMethodID};
use jni::JNIEnv;
use std::sync::Mutex;

macro_rules! cache_ref {
    ($name:ident : $ty:ty) => { paste::paste! {
		#[allow(non_upper_case_globals)]
		static [<INNER_ $name>]: Mutex<Option<$ty>> = Mutex::new(None);

		/// Unwraps this cache member as long as `JNI_OnUnload` has not been called yet.
		/// The values returned by this should be disposed of as quickly as possible and not held.
		#[allow(non_snake_case)]
		pub(crate) fn $name() -> $ty
			where $ty: Clone
		{
			[<INNER_ $name>].lock()
				.expect("jni_cache mutex lock fail")
				.as_ref()
				.expect("JNI cache already cleaned up")
				.clone()
		}

		/// Initializes this global cached value. If it already contains a value, a panic occurs.
		#[allow(non_snake_case)]
		fn [<init_ $name>](value: $ty)
			where $ty: Clone
		{
			let mut option = [<INNER_ $name>].lock()
				.expect("jni_cache mutex lock fail");

			match option.as_ref() {
				Some(_) => panic!("jni_cache member already initialized"),
				None => { *option = Some(value); }
			};
		}
	}};
}

macro_rules! clear_refs {
	($($name:ident,)*) => { paste::paste! {
		$(
		[<INNER_ $name>].lock()
		.expect("jni_cache mutex lock fail")
		.take();
		)*
	}};
}

cache_ref!(InvalidArgumentException: GlobalRef);
cache_ref!(RuntimeException: GlobalRef);
cache_ref!(NativeCallbacks: GlobalRef);
cache_ref!(onResponse: JMethodID);
cache_ref!(onError: JMethodID);

#[catch_panic(default = "false")]
pub(super) fn init_cache(mut env: JNIEnv) -> bool {
	fn class_ref(env: &mut JNIEnv, name: &str) -> GlobalRef {
		env.find_class(name)
			.and_then(|cls| env.new_global_ref(cls))
			.expect("failed to get class for jni_cache member")
	}

	init_InvalidArgumentException(class_ref(&mut env, "java/lang/InvalidArgumentException"));
	init_RuntimeException(class_ref(&mut env, "java/lang/RuntimeException"));
	init_NativeCallbacks(class_ref(&mut env, "dev/rushii/ktor_impersonate/Native$Callbacks"));
	init_onResponse(env.get_method_id(&NativeCallbacks(), "onResponse", "(ILjava/lang/String;)V").unwrap());
	init_onError(env.get_method_id(&NativeCallbacks(), "onError", "(Ljava/lang/String;)V").unwrap());
	true
}

/// Class refs should be deleted after all their member IDs have been.
/// Otherwise, if the class gets unloaded by the JVM, all the method/field IDs become invalid.s
#[catch_panic(default = "false")]
pub(super) fn release_cache(env: JNIEnv) -> bool {
	clear_refs!(
		InvalidArgumentException,
		RuntimeException,

		onResponse,
		onError,
		NativeCallbacks,
	);

	true
}
