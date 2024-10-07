# ktor-impersonate (WIP)

### This project is currently under development and is not ready for usage!

A Kotlin-Multiplatform (KMP)<sup>1</sup> Ktor client engine that binds to the [rquest] (aka. reqwest-impersonate) Rust crate,
allowing configuring the TLS `ClientHello` and HTTP/2 options at a low level.

This allows impersonating browsers and other HTTP clients with ease, and spoofing JA3/JA4/Akamai HTTP/2 fingerprints.
Presets are provided for convenience of Chrome, Safari, Edge, and OkHTTP.


<sup>1:
Currently only Android (x86, x86_64, armeabi-v7a, arm64-v8a) is supported.
More platforms, including JVM, Desktop native, and iOS, will be supported at a later date.
</sup>
<br/>

# Usage

`build.gradle.kts`:
```kts
val version = "1.0.0"

// Without KMP
dependencies {
	implementation("dev.rushii.ktor-impersonate:ktor-impersonate:$version")
}

// KMP
kotlin {
	sourceSets {
		commonMain.dependencies {
			implementation("dev.rushii.ktor-impersonate:ktor-impersonate:$version")
		}
	}
}
```

Quick start:
```kotlin
import dev.rushii.ktor_impersonate.*
import io.ktor.client.*
import io.ktor.client.request.*

val client = HttpClient(Impersonate) {
  engine {
    preset = ImpersonatePreset.Chrome129
  }
}

client.get("https://google.com")
```

[rquest]: https://github.com/penumbra-x/rquest
