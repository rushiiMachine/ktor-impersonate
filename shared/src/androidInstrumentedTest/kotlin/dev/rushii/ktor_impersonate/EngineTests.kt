package dev.rushii.ktor_impersonate

import androidx.test.ext.junit.runners.AndroidJUnit4
import io.ktor.client.HttpClient
import io.ktor.client.request.get
import kotlinx.coroutines.runBlocking
import org.junit.Test
import org.junit.runner.RunWith

/**
 * These tests run on an Android device (or emulator).
 * The architecture of the native library should match the device.
 */
@RunWith(AndroidJUnit4::class)
class EngineTests {
	@Test
	fun loadNativeLibrary() {
		initializeNative()
	}

	@Test
	fun createEngine() {
		ImpersonateEngine(ImpersonateConfig()).apply {
			close()
			close() // Shouldn't crash
		}
	}

	@Test
	fun createClient() {
		HttpClient(Impersonate).close()
	}

	@Test
	fun sendHttpRequest() {
		val client = HttpClient(Impersonate)

		runBlocking {
			client.get("http://example.com/")
		}
	}

	@Test
	fun sendHttpsRequest() {
		val client = HttpClient(Impersonate)

		runBlocking {
			client.get("https://example.com/")
		}
	}
}
