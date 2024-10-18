package dev.rushii.ktor_impersonate.internal

import dev.rushii.ktor_impersonate.ImpersonateConfig
import io.ktor.http.Headers
import kotlin.jvm.JvmStatic

/**
 * Loads and initializes the native portion of this lib
 */
internal expect fun initializeNative()

@Suppress("unused")
internal object NativeEngine {
	@JvmStatic
	external fun createClient(config: ImpersonateConfig): Long

	@JvmStatic
	external fun destroyClient(clientPtr: Long)

	@JvmStatic
	external fun executeRequest(
		clientPtr: Long,
		callbacks: Callbacks,
		url: String,
		httpMethod: String,
		headers: Headers,
		isWebsocket: Boolean,
	): Int

	@JvmStatic
	external fun cancelRequest(requestId: Int)

	abstract class Callbacks {
		abstract fun onResponse(version: String, code: Int, headers: Headers)
		abstract fun onError(message: String)
	}
}
