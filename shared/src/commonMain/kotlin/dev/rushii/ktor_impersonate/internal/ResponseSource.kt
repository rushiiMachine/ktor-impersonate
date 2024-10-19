package dev.rushii.ktor_impersonate.internal

import kotlinx.io.Buffer
import kotlinx.io.RawSource

/**
 * Collects the response body of a currently active request from the native side.
 */
internal class ResponseSource(
	/** Used by the native side */
	@Suppress("unused")
	private val requestId: Int,
) : RawSource {
	external fun init()
	external override fun close()
	external override fun readAtMostTo(sink: Buffer, byteCount: Long): Long

	init {
		init()
	}
}
