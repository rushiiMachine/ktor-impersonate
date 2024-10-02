package dev.rushii.ktor_impersonate

// TODO: Throw different kinds of exceptions based on the actual error type.
//       For example, throw IOException for connection errors, etc.

/**
 * Exception type wrapping errors returned by the native rquest library,
 * including instances requests cannot complete successfully.
 * The [message] will contain the error kind, url (if applicable), and an underlying error (if applicable).
 */
public class RquestException internal constructor(override val message: String) : Exception()
