@file:OptIn(ExperimentalKotlinGradlePluginApi::class)

import com.android.build.gradle.tasks.MergeSourceSetFolders
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
}

tasks.getByName<Delete>("clean") {
	delete("../rust/target")
}
