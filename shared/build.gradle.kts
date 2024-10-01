@file:OptIn(ExperimentalKotlinGradlePluginApi::class)

import com.android.build.gradle.tasks.MergeSourceSetFolders
import org.gradle.kotlin.dsl.support.listFilesOrdered
import org.jetbrains.kotlin.gradle.ExperimentalKotlinGradlePluginApi
import org.jetbrains.kotlin.gradle.dsl.JvmTarget
import org.jetbrains.kotlin.gradle.plugin.mpp.apple.XCFramework

plugins {
//	alias(libs.plugins.binaryCompatibilityValidator)
	alias(libs.plugins.kotlinMultiplatform)
	alias(libs.plugins.androidLibrary)
	alias(libs.plugins.rustAGP)
}

kotlin {
	explicitApi()

	sourceSets {
		commonMain.dependencies {
			implementation(libs.ktor)
		}
		commonTest.dependencies {
			implementation(libs.kotlin.test)
		}
	}

	val iosTargets = listOf(iosX64(), iosArm64(), iosSimulatorArm64())
	val xcf = project.XCFramework()
	for (target in iosTargets) {
		target.binaries.framework {
			baseName = "lib"
			isStatic = true
			xcf.add(this@framework)
		}
	}

	androidTarget {
		compilerOptions {
			jvmTarget.set(JvmTarget.JVM_1_8)
		}
	}
}

android {
	namespace = "dev.rushii.ktor_impersonate"
	compileSdk = 34

	defaultConfig {
		minSdk = 21
	}

	compileOptions {
		sourceCompatibility = JavaVersion.VERSION_1_8
		targetCompatibility = JavaVersion.VERSION_1_8
	}

	ndkVersion = sdkDirectory.resolve("ndk").listFilesOrdered().last().name
}

cargo {
	module = "../rust"
	libname = "ktorimpersonate"
	targets = listOf("arm", "arm64", "x86", "x86_64")
	prebuiltToolchains = true

	// This assumes that either `assembleDebug` or `assembleRelease` is used to build
	gradle.taskGraph.whenReady {
		if (allTasks.any { it.name.contains("release") })
			this@cargo.profile = "release"
	}
}

afterEvaluate {
	for (name in arrayOf("mergeDebugJniLibFolders", "mergeReleaseJniLibFolders")) {
		tasks.getByName(name, MergeSourceSetFolders::class) {
			dependsOn("cargoBuild")
			inputs.dir(layout.buildDirectory.asFile.get().resolve("rustJniLibs/android"))
		}
	}

	// Set environment variables for boring ssl compilation targeting the Android NDK
	// Carry it through properties for the rust-android-gradle-plugin
	for (targetTriple in arrayOf("armv7-linux-androideabi", "aarch64-linux-android", "i686-linux-android", "x86_64-linux-android")) {
		val target = targetTriple.uppercase().replace('-', '_')

		// Set ANDROID_NDK_HOME pointing to latest NDK toolchain
		project.ext.set("RUST_ANDROID_GRADLE_TARGET_${target}_ANDROID_NDK_HOME", android.ndkDirectory.absolutePath)

		// Set CMAKE_GENERATOR to make cmake wrapper crate pass through the proper generator
		project.ext.set("RUST_ANDROID_GRADLE_TARGET_${target}_CMAKE_GENERATOR", "Ninja")

		// Add Android SDK's cmake install to PATH to use that cmake & ninja build
		val pathSeparator = if (System.getenv("OS").contains("windows", ignoreCase = true)) ";" else ":"
		val cmakeDir = android.sdkDirectory.resolve("cmake").listFilesOrdered().last().resolve("bin").absolutePath
		project.ext.set("RUST_ANDROID_GRADLE_TARGET_${target}_PATH", cmakeDir + pathSeparator + System.getenv("PATH"))
	}
}

tasks.getByName<Delete>("clean") {
	delete("../rust/target")
}
