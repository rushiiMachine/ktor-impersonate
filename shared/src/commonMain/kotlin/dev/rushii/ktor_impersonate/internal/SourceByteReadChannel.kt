package dev.rushii.ktor_impersonate.internal

import io.ktor.utils.io.ByteReadChannel
import io.ktor.utils.io.InternalAPI
import io.ktor.utils.io.core.endOfInput
import io.ktor.utils.io.core.remaining
import kotlinx.io.IOException
import kotlinx.io.Source
import kotlin.concurrent.Volatile

internal class SourceByteReadChannel(private val source: Source) : ByteReadChannel {
	@Volatile
	override var closedCause: Throwable? = null

	override val isClosedForRead: Boolean
		get() = source.exhausted()

	@InternalAPI
	override val readBuffer: Source
		get() {
			closedCause?.let { throw it }
			return source
		}

	override suspend fun awaitContent(min: Int): Boolean {
		closedCause?.let { throw it }
		return source.request(min.toLong())
	}

	override fun cancel(cause: Throwable?) {
		if (closedCause != null) return
		source.close()
		closedCause = IOException(cause?.message ?: "Channel was cancelled", cause)
	}
}
