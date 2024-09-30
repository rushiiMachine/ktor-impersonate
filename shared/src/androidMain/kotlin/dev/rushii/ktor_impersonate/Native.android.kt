package dev.rushii.ktor_impersonate

internal actual fun initializeNative() {
	System.loadLibrary("ktorimpersonate")
}
