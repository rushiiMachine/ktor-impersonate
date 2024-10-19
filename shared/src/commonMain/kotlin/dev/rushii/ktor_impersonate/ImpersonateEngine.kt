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
import kotlinx.io.*
import kotlin.coroutines.*

public class ImpersonateEngine(override val config: ImpersonateConfig) : HttpClientEngineBase("ktor-impersonate") {
	// Pointer to the native rquest client.
	private var nativeClientPtr: Long = NativeEngine.createClient(config)

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
			val callbacks = object : NativeEngine.Callbacks() {
				override fun onResponse(version: String, code: Int, headers: Headers) {
					try {
						val data = HttpResponseData(
							statusCode = HttpStatusCode.fromValue(code),
							requestTime = requestTime,
							headers = headers,
							version = HttpProtocolVersion.parse(version),
							body = SourceByteReadChannel(ResponseSource(requestId).buffered()),
							callContext = callContext,
						)
						continuation.resume(data)
					} catch (t: Throwable) {
						continuation.resumeWithException(t)
					}
				}

				override fun onError(message: String) {
					continuation.resumeWithException(RquestException(message))
				}
			}

			// Start native request
			requestId = NativeEngine.executeRequest(
				clientPtr = nativeClientPtr,
				callbacks = callbacks,
				url = data.url.toString(),
				httpMethod = data.method.value,
				headers = data.headers,
				isWebsocket = data.isUpgradeRequest(),
			)

			// Abort native request if coroutine gets cancelled
			continuation.invokeOnCancellation { NativeEngine.cancelRequest(requestId) }

			// Cancel native request if coroutine cancelled before the cancellation handler was registered
			if (continuation.isCancelled) NativeEngine.cancelRequest(requestId)
		}
	}

	override fun close() {
		super.close()
		val ptr = nativeClientPtr
		nativeClientPtr = 0
		NativeEngine.destroyClient(ptr)
	}

	// Sigh... if only kotlin had static initializer blocks
	private companion object {
		init {
			initializeNative()
		}
	}
}
