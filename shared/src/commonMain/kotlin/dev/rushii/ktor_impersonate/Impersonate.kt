package dev.rushii.ktor_impersonate

import io.ktor.client.engine.HttpClientEngine
import io.ktor.client.engine.HttpClientEngineFactory
import io.ktor.client.plugins.HttpTimeout

/**
 * An Android client engine that binds to the Rust crate [rquest](https://crates.io/crates/rquest)
 * (formerly known as reqwest-impersonate) in order to spoof TLS/JA3/JA4/JA4/HTTP2 fingerprints.
 *
 * Quick start:
 * ```kotlin
 * val client = HttpClient(Impersonate) {
 *   engine {
 *     impersonate(ImpersonatePreset.Chrome129) // Uses a preset
 *   }
 * }
 * ```
 *
 * **Notes**:
 * - Any changes to the engine configuration will be ignored once the engine has been initialized.
 * - SSE is not supported
 * - [HttpTimeout]'s only working setting is [HttpTimeout.requestTimeoutMillis]
 */
public object Impersonate : HttpClientEngineFactory<ImpersonateConfig> {
	override fun create(block: ImpersonateConfig.() -> Unit): HttpClientEngine {
		return ImpersonateEngine(ImpersonateConfig().apply(block))
	}
}
