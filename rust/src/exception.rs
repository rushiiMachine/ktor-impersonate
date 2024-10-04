/// Throw a `IllegalArgumentException` and early return with/without a value.
/// Examples:
/// ```rs
/// let env: &mut JNIEnv = ...;
/// throw_argument(env, "size cannot be 0");
/// throw_argument(env, "builder cannot be null", std::ptr::null_mut());
/// ```
#[macro_export]
macro_rules! throw_argument {
	($env:ident, $msg:expr) => {{
		$env.throw_new(&crate::jni_cache::IllegalArgumentException(), $msg).expect("failed to throw error");
		return;
	}};
    ($env:ident, $msg:expr, $ret:expr) => {{
		$env.throw_new(&crate::jni_cache::IllegalArgumentException(), $msg).expect("failed to throw error");
		return $ret;
	}};
}

/// Throw an `RuntimeException` exception early return with/without a value.
/// Refer to [throw_argument] for usage examples.
#[macro_export]
macro_rules! throw {
	($env:ident, $msg:expr) => {{
		$env.throw_new(&crate::jni_cache::RuntimeException(), $msg).expect("failed to throw error");
		return;
	}};
    ($env:ident, $msg:expr, $ret:expr) => {{
		$env.throw_new(&crate::jni_cache::RuntimeException(), $msg).expect("failed to throw error");
		return $ret;
	}};
}
