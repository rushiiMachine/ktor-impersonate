package dev.rushii.ktor_impersonate

import io.ktor.client.engine.HttpClientEngineConfig
import io.ktor.client.plugins.HttpTimeoutConfig
import kotlin.jvm.JvmName
import kotlin.time.Duration

/**
 * Opt-in marker for APIs that allow for the breaking of the chain of trust.
 */
@RequiresOptIn
public annotation class DangerousHTTPApi

/**
 * Config for setting up an Impersonate client
 */
public class ImpersonateConfig : HttpClientEngineConfig() {
	/**
	 * Enable verbose connection logging to the platform-specific application logs at the TRACE/verbose level.
	 */
	public var verboseLogging: Boolean = false

	/**
	 * Apply a preset of TLS, HTTP 2.0, and default headers mimicking a specific browser or HTTP client.
	 * This will be applied first before any other config options.
	 * Preset info can be viewed [here](https://github.com/penumbra-x/rqeust/tree/c69d54e9bcc972280b244e302b0e5751f74d6f88/src/tls/impersonate).
	 */
	@get:JvmName("getPreset")
	public var preset: ImpersonatePreset? = null

	// =========== Timeout options =========== //

	/**
	 * Enables a request timeout on all requests made from this client.
	 * The timeout is applied from when the request starts connecting until the response body has finished.
	 * Default is no timeout.
	 *
	 * **Note:** [HttpTimeoutConfig.requestTimeoutMillis] is separate from this timeout and is applied separately.
	 */
	public var requestTimeout: Duration? = null

	/**
	 * Enables a timeout for only a new socket connection.
	 * This should be smaller than [requestTimeout] significantly.
	 * Default is no timeout.
	 *
	 * **Note:** [HttpTimeoutConfig.connectTimeoutMillis] does nothing.
	 */
	public var connectTimeout: Duration? = null

	/**
	 * Set a timeout for idle socket connections being kept-alive.
	 * Once expired, connection will be closed.
	 * Default is 90 seconds.
	 *
	 * **Note:** This is different from the HTTP 2.0 keep alive mechanism.
	 * **Note:** [HttpTimeoutConfig.socketTimeoutMillis] does nothing.
	 */
	public var idleTimeout: Duration? = null

	// =========== TLS options =========== //

	/**
	 * Disables certificate validation.
	 *
	 * ## CAUTION
	 * ANY certificate will be trusted, including expired, self-signed, and forged ones.
	 * Use only if you actually know what you're doing.
	 */
	@DangerousHTTPApi
	public var allowInvalidCertificates: Boolean? = null

//	/**
//	 * Sets the CA certificate from a file.
//	 * This can be used to allow self-signed certificates.
//	 */
//	@DangerousHTTPApi
//	public var rootCertificates: List<Any>? = null

	// =========== HTTP options =========== //

	// =========== HTTPS options =========== //

	/**
	 * Restricts the client to only send HTTPS requests.
	 * Defaults to false.
	 */
	public var httpsOnly: Boolean? = null

	// =========== HTTP/2 options =========== //

	// =========== Internal =========== //

	// Internal methods used by native code
	// kotlin.time.Duration (an inline class) is hard to use due to name mangling

	// @formatter:off
	@Suppress("unused") private fun getRequestTimeoutMillis(): Long? = requestTimeout?.inWholeMilliseconds
	@Suppress("unused") private fun getConnectTimeoutMillis(): Long? = connectTimeout?.inWholeMilliseconds
	@Suppress("unused") private fun getIdleTimeout(): Long? = idleTimeout?.inWholeMilliseconds
	// @formatter:on
}
