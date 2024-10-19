use catch_panic::catch_panic;
use jni::objects::{GlobalRef, JFieldID, JMethodID};
use jni::JNIEnv;
use std::sync::Mutex;

macro_rules! cache_ref {
    ($name:ident : $ty:ty) => { paste::paste! {
		#[allow(non_upper_case_globals)]
		static [<INNER_ $name>]: Mutex<Option<$ty>> = Mutex::new(None);

		/// Unwraps this cache member as long as `JNI_OnUnload` has not been called yet.
		/// The values returned by this should be disposed of as quickly as possible and not held.
		#[allow(non_snake_case)]
		pub fn $name() -> $ty
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

// Java Stdlib
cache_ref!(Boolean: GlobalRef);
cache_ref!(Boolean_booleanValue: JMethodID);
cache_ref!(Class: GlobalRef);
cache_ref!(Class_getName: JMethodID);
cache_ref!(IllegalArgumentException: GlobalRef);
cache_ref!(List: GlobalRef);
cache_ref!(List_toArray: JMethodID);
cache_ref!(Long: GlobalRef);
cache_ref!(Long_longValue: JMethodID);
cache_ref!(RuntimeException: GlobalRef);
cache_ref!(Set: GlobalRef);
cache_ref!(Set_toArray: JMethodID);

// Kotlinx
cache_ref!(Sink: GlobalRef);
cache_ref!(Sink_write: JMethodID);

// ktor-impersonate
cache_ref!(ImpersonateConfig: GlobalRef);
cache_ref!(ImpersonateConfig_getVerboseLogging: JMethodID);
cache_ref!(ImpersonateConfig_getPreset: JMethodID);
cache_ref!(ImpersonateConfig_getRequestTimeoutMillis: JMethodID);
cache_ref!(ImpersonateConfig_getConnectTimeoutMillis: JMethodID);
cache_ref!(ImpersonateConfig_getIdleTimeout: JMethodID);
cache_ref!(ImpersonateConfig_getAllowInvalidCertificates: JMethodID);
cache_ref!(ImpersonateConfig_getHttpsOnly: JMethodID);
cache_ref!(NativeCallbacks: GlobalRef);
cache_ref!(NativeCallbacks_onError: JMethodID);
cache_ref!(NativeCallbacks_onResponse: JMethodID);
cache_ref!(ResponseSource: GlobalRef);
cache_ref!(ResponseSource_requestId: JFieldID);

// Ktor
cache_ref!(HeadersBuilder: GlobalRef);
cache_ref!(HeadersBuilder_build: JMethodID);
cache_ref!(HeadersBuilder_init: JMethodID);
cache_ref!(StringValues: GlobalRef);
cache_ref!(StringValues_getAll: JMethodID);
cache_ref!(StringValues_names: JMethodID);
cache_ref!(StringValuesBuilder: GlobalRef);
cache_ref!(StringValuesBuilder_append: JMethodID);

#[catch_panic(default = "false")]
pub(super) fn init_cache(mut env: JNIEnv) -> bool {
	fn class_ref(env: &mut JNIEnv, name: &str) -> GlobalRef {
		env.find_class(name)
			.and_then(|cls| env.new_global_ref(cls))
			.expect("failed to get class for jni_cache member")
	}

	// TODO: better errors when failing to unwrap so users can figure out proguard issues

	// Java Stdlib
	init_Boolean(class_ref(&mut env, "java/lang/Boolean"));
	init_Boolean_booleanValue(env.get_method_id(&Boolean(), "booleanValue", "()Z").unwrap());
	init_Class(class_ref(&mut env, "java/lang/Class"));
	init_Class_getName(env.get_method_id(&Class(), "getName", "()Ljava/lang/String;").unwrap());
	init_IllegalArgumentException(class_ref(&mut env, "java/lang/IllegalArgumentException"));
	init_List(class_ref(&mut env, "java/util/List"));
	init_List_toArray(env.get_method_id(&List(), "toArray", "()[Ljava/lang/Object;").unwrap());
	init_Long(class_ref(&mut env, "java/lang/Long"));
	init_Long_longValue(env.get_method_id(&Long(), "longValue", "()J").unwrap());
	init_RuntimeException(class_ref(&mut env, "java/lang/RuntimeException"));
	init_Set(class_ref(&mut env, "java/util/Set"));
	init_Set_toArray(env.get_method_id(&Set(), "toArray", "()[Ljava/lang/Object;").unwrap());

	// Kotlinx
	init_Sink(class_ref(&mut env, "kotlinx/io/Sink"));
	init_Sink_write(env.get_method_id(&Sink(), "write", "([BII)V").unwrap());

	// ktor-impersonate
	init_ImpersonateConfig(class_ref(&mut env, "dev/rushii/ktor_impersonate/ImpersonateConfig"));
	init_ImpersonateConfig_getVerboseLogging(env.get_method_id(&ImpersonateConfig(), "getVerboseLogging", "()Z").unwrap());
	init_ImpersonateConfig_getPreset(env.get_method_id(&ImpersonateConfig(), "getPreset", "()Ljava/lang/String;").unwrap());
	init_ImpersonateConfig_getRequestTimeoutMillis(env.get_method_id(&ImpersonateConfig(), "getRequestTimeoutMillis", "()Ljava/lang/Long;").unwrap());
	init_ImpersonateConfig_getConnectTimeoutMillis(env.get_method_id(&ImpersonateConfig(), "getConnectTimeoutMillis", "()Ljava/lang/Long;").unwrap());
	init_ImpersonateConfig_getIdleTimeout(env.get_method_id(&ImpersonateConfig(), "getIdleTimeout", "()Ljava/lang/Long;").unwrap());
	init_ImpersonateConfig_getAllowInvalidCertificates(env.get_method_id(&ImpersonateConfig(), "getAllowInvalidCertificates", "()Ljava/lang/Boolean;").unwrap());
	init_ImpersonateConfig_getHttpsOnly(env.get_method_id(&ImpersonateConfig(), "getHttpsOnly", "()Ljava/lang/Boolean;").unwrap());
	init_NativeCallbacks(class_ref(&mut env, "dev/rushii/ktor_impersonate/internal/NativeEngine$Callbacks"));
	init_NativeCallbacks_onError(env.get_method_id(&NativeCallbacks(), "onError", "(Ljava/lang/String;)V").unwrap());
	init_NativeCallbacks_onResponse(env.get_method_id(&NativeCallbacks(), "onResponse", "(Ljava/lang/String;ILio/ktor/http/Headers;)V").unwrap());
	init_ResponseSource(class_ref(&mut env, "dev/rushii/ktor_impersonate/internal/ResponseSource"));
	init_ResponseSource_requestId(env.get_field_id(&ResponseSource(), "requestId", "I").unwrap());

	// Ktor
	init_HeadersBuilder(class_ref(&mut env, "io/ktor/http/HeadersBuilder"));
	init_HeadersBuilder_build(env.get_method_id(&HeadersBuilder(), "build", "()Lio/ktor/http/Headers;").unwrap());
	init_HeadersBuilder_init(env.get_method_id(&HeadersBuilder(), "<init>", "(I)V").unwrap());
	init_StringValues(class_ref(&mut env, "io/ktor/util/StringValues"));
	init_StringValues_getAll(env.get_method_id(&StringValues(), "getAll", "(Ljava/lang/String;)Ljava/util/List;").unwrap());
	init_StringValues_names(env.get_method_id(&StringValues(), "names", "()Ljava/util/Set;").unwrap());
	init_StringValuesBuilder(class_ref(&mut env, "io/ktor/util/StringValuesBuilder"));
	init_StringValuesBuilder_append(env.get_method_id(&StringValuesBuilder(), "append", "(Ljava/lang/String;Ljava/lang/String;)V").unwrap());
	true
}

/// Release all the [`GlobalRef`]s from this cache.
// GlobalRefs of classes should be deleted after all the member IDs for that class have been.
// Otherwise, when the class gets unloaded by the JVM, all the method/field IDs become invalid.
pub(super) fn release_cache() {
	clear_refs!(
		// Java Stdlib
		Class_getName,
		Class,
		IllegalArgumentException,
		RuntimeException,
		List_toArray,
		List,
		Long_longValue,
		Long,
		Set_toArray,
		Set,

		// Kotlinx
		Sink_write,
		Sink,

		// ktor-impersonate
		ImpersonateConfig_getVerboseLogging,
		ImpersonateConfig_getPreset,
		ImpersonateConfig_getRequestTimeoutMillis,
		ImpersonateConfig_getConnectTimeoutMillis,
		ImpersonateConfig_getIdleTimeout,
		ImpersonateConfig_getAllowInvalidCertificates,
		ImpersonateConfig_getHttpsOnly,
		ImpersonateConfig,
		NativeCallbacks_onError,
		NativeCallbacks_onResponse,
		NativeCallbacks,
		ResponseSource_requestId,
		ResponseSource,

		// Ktor
		HeadersBuilder_build,
		HeadersBuilder_init,
		HeadersBuilder,
		StringValues_getAll,
		StringValues_names,
		StringValues,
		StringValuesBuilder_append,
		StringValuesBuilder,
	);
}
