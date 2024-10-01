package dev.rushii.ktor_impersonate

import kotlin.jvm.JvmStatic

/**
 * Loads and initializes the native portion of this lib
 */
internal expect fun initializeNative()

@Suppress("unused")
internal object Native {
	@JvmStatic
	external fun createClient(): Long
	@JvmStatic
	external fun destroyClient(clientPtr: Long)
	@JvmStatic
	external fun executeRequest(clientPtr: Long, callbacks: Callbacks, url: String, httpMethod: String, isWebsocket: Boolean): Int
	@JvmStatic
	external fun cancelRequest(requestId: Int)

	open class Callbacks {
		open fun onResponse(code: Int, version: String) {}
	}
}
