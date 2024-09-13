@file:OptIn(ExperimentalKotlinGradlePluginApi::class)

import org.jetbrains.kotlin.gradle.ExperimentalKotlinGradlePluginApi
import org.jetbrains.kotlin.gradle.dsl.JvmTarget
import org.jetbrains.kotlin.gradle.plugin.mpp.apple.XCFramework

plugins {
//	alias(libs.plugins.binaryCompatibilityValidator)
	alias(libs.plugins.kotlinMultiplatform)
	alias(libs.plugins.androidLibrary)
}

kotlin {
	explicitApi()

	sourceSets {
		commonMain.dependencies {}
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
}
