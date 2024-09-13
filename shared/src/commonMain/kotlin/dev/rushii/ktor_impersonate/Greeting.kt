package dev.rushii.ktor_impersonate

public class Greeting {
	private val platform: Platform = getPlatform()

	public fun greet(): String {
		return "Hello, ${platform.name}!"
	}
}
