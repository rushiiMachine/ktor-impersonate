@file:Suppress("FunctionName")

package dev.rushii.ktor_impersonate

import dev.rushii.ktor_impersonate.internal.*
import io.ktor.client.engine.*
import io.ktor.client.plugins.websocket.*
import io.ktor.client.request.*
import io.ktor.http.*
import io.ktor.util.date.*
import io.ktor.utils.io.*
import kotlinx.coroutines.*
import kotlin.coroutines.*

public class ImpersonateEngine(override val config: ImpersonateConfig) : HttpClientEngineBase("ktor-impersonate") {
	// Pointer to the native rquest client.
	private var nativeClientPtr: Long = Native.createClient(config)

	// Reqwest does not support SSE
	override val supportedCapabilities: Set<HttpClientEngineCapability<*>>
		get() = setOf(WebSocketCapability, WebSocketExtensionsCapability)

	@OptIn(InternalAPI::class)
	override suspend fun execute(data: HttpRequestData): HttpResponseData {
		val callContext = callContext()
		val requestTime = GMTDate()

		return suspendCancellableCoroutine { continuation ->
			var requestId: Int = 0

			// Make callbacks to handle native request completion
			val callbacks = object : Native.Callbacks() {
				override fun onResponse(version: String, code: Int, headers: Headers) {
					val data = HttpResponseData(
						statusCode = HttpStatusCode.fromValue(code),
						requestTime = requestTime,
						headers = headers,
						version = HttpProtocolVersion.parse(version),
						body = "", // TODO: this
						callContext = callContext,
					)
					continuation.resume(data)
				}

				override fun onError(message: String) {
					continuation.resumeWithException(RquestException(message))
				}
			}

			// Start native request
			requestId = Native.executeRequest(
				clientPtr = nativeClientPtr,
				callbacks = callbacks,
				url = data.url.toString(),
				httpMethod = data.method.value,
				headers = data.headers,
				isWebsocket = data.isUpgradeRequest(),
			)

			// Abort native request if coroutine gets cancelled
			continuation.invokeOnCancellation { Native.cancelRequest(requestId) }

			// Cancel native request if coroutine cancelled before the cancellation handler was registered
			if (continuation.isCancelled) Native.cancelRequest(requestId)
		}
	}

	override fun close() {
		super.close()
		val ptr = nativeClientPtr
		nativeClientPtr = 0
		Native.destroyClient(ptr)
	}

	// Sigh... if only kotlin had static initializer blocks
	private companion object {
		init {
			initializeNative()
		}
	}
}
