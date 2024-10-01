package dev.rushii.ktor_impersonate

import io.ktor.client.engine.HttpClientEngineConfig

/**
 * Config for setting up an Impersonate client
 */
public class ImpersonateConfig : HttpClientEngineConfig() {
	/**
	 * Apply a preset of TLS, HTTP 2.0, and default headers mimicking a specific browser or HTTP client.
	 * This will be applied first before any other config options.
	 * Preset info can be viewed [here](https://github.com/penumbra-x/rqeust/tree/c69d54e9bcc972280b244e302b0e5751f74d6f88/src/tls/impersonate).
	 */
	public var preset: ImpersonatePreset? = null
}
