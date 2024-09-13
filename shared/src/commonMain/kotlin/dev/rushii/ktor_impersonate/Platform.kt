package dev.rushii.ktor_impersonate

internal interface Platform {
	val name: String
}

internal expect fun getPlatform(): Platform
