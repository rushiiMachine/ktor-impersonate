use catch_panic::catch_panic;
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

#[catch_panic(default = "false")]
pub(super) unsafe fn init_cache(mut env: JNIEnv) -> bool {
	true
}

/// Class refs should be deleted after all their member IDs have been.
/// Otherwise, if the class gets unloaded by the JVM, all the method/field IDs become invalid.s
#[catch_panic(default = "false")]
pub(super) unsafe fn release_cache(env: JNIEnv) -> bool {
	true
}
