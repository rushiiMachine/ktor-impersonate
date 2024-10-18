package dev.rushii.ktor_impersonate.internal

internal actual fun initializeNative() {
	System.loadLibrary("ktorimpersonate")
}
