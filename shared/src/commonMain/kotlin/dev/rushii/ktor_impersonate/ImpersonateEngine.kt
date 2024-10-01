@file:Suppress("FunctionName")

package dev.rushii.ktor_impersonate

import io.ktor.client.engine.*
import io.ktor.client.plugins.websocket.*
import io.ktor.client.request.*
import io.ktor.http.*
import io.ktor.util.*
import io.ktor.util.date.*
import kotlinx.coroutines.*
import kotlin.coroutines.resume

@OptIn(InternalAPI::class)
public class ImpersonateEngine(override val config: ImpersonateConfig) : HttpClientEngineBase("ktor-impersonate") {
	// Pointer to the native rquest client.
	private var nativeClientPtr: Long = Native.createClient(config)

	// Reqwest does not support SSE
	override val supportedCapabilities: Set<HttpClientEngineCapability<*>>
		get() = setOf(WebSocketCapability, WebSocketExtensionsCapability)

	override suspend fun execute(data: HttpRequestData): HttpResponseData {
		val callContext = callContext()
		val requestTime = GMTDate()

		return suspendCancellableCoroutine { continuation ->
			var requestId: Int = 0

			// Make callbacks to handle native request completion
			val callbacks = object : Native.Callbacks() {
				override fun onResponse(code: Int, version: String) {
					val data = HttpResponseData(
						statusCode = HttpStatusCode.fromValue(code),
						requestTime = requestTime,
						headers = Headers.Empty, // TODO: this
						version = HttpProtocolVersion.parse(version),
						body = "", // TODO: this
						callContext = callContext,
					)
					continuation.resume(data)
				}
			}

			// Start request
			requestId = Native.executeRequest(
				clientPtr = nativeClientPtr,
				callbacks = callbacks,
				url = data.url.toString(),
				httpMethod = data.method.value,
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
