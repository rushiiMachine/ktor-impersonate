@file:Suppress("ConstPropertyName")

package dev.rushii.ktor_impersonate

import kotlin.jvm.JvmInline

/**
 * Preconfigured TLS, HTTP 2.0, and default headers mimicking a specific browser or HTTP client.
 * Preset info can be viewed [here](https://github.com/penumbra-x/rqeust/tree/c69d54e9bcc972280b244e302b0e5751f74d6f88/src/tls/impersonate).
 */
@JvmInline
public value class ImpersonatePreset private constructor(private val presetName: String) {
	public companion object {
		// Chrome
		public val Chrome100: ImpersonatePreset get() = ImpersonatePreset("chrome_100")
		public val Chrome101: ImpersonatePreset get() = ImpersonatePreset("chrome_101")
		public val Chrome104: ImpersonatePreset get() = ImpersonatePreset("chrome_104")
		public val Chrome105: ImpersonatePreset get() = ImpersonatePreset("chrome_105")
		public val Chrome106: ImpersonatePreset get() = ImpersonatePreset("chrome_106")
		public val Chrome107: ImpersonatePreset get() = ImpersonatePreset("chrome_107")
		public val Chrome108: ImpersonatePreset get() = ImpersonatePreset("chrome_108")
		public val Chrome109: ImpersonatePreset get() = ImpersonatePreset("chrome_109")
		public val Chrome114: ImpersonatePreset get() = ImpersonatePreset("chrome_114")
		public val Chrome116: ImpersonatePreset get() = ImpersonatePreset("chrome_116")
		public val Chrome117: ImpersonatePreset get() = ImpersonatePreset("chrome_117")
		public val Chrome118: ImpersonatePreset get() = ImpersonatePreset("chrome_118")
		public val Chrome119: ImpersonatePreset get() = ImpersonatePreset("chrome_119")
		public val Chrome120: ImpersonatePreset get() = ImpersonatePreset("chrome_120")
		public val Chrome123: ImpersonatePreset get() = ImpersonatePreset("chrome_123")
		public val Chrome124: ImpersonatePreset get() = ImpersonatePreset("chrome_124")
		public val Chrome126: ImpersonatePreset get() = ImpersonatePreset("chrome_126")
		public val Chrome127: ImpersonatePreset get() = ImpersonatePreset("chrome_127")
		public val Chrome128: ImpersonatePreset get() = ImpersonatePreset("chrome_128")
		public val Chrome129: ImpersonatePreset get() = ImpersonatePreset("chrome_129")

		// Safari
		public val SafariIos17_2: ImpersonatePreset get() = ImpersonatePreset("safari_ios_17.2")
		public val SafariIos17_4_1: ImpersonatePreset get() = ImpersonatePreset("safari_ios_17.4.1")
		public val Safari15_3: ImpersonatePreset get() = ImpersonatePreset("safari_15.3")
		public val Safari15_5: ImpersonatePreset get() = ImpersonatePreset("safari_15.5")
		public val Safari15_6_1: ImpersonatePreset get() = ImpersonatePreset("safari_15.6.1")
		public val Safari16: ImpersonatePreset get() = ImpersonatePreset("safari_16")
		public val Safari16_5: ImpersonatePreset get() = ImpersonatePreset("safari_16.5")
		public val SafariIos16_5: ImpersonatePreset get() = ImpersonatePreset("safari_ios_16.5")
		public val Safari17_0: ImpersonatePreset get() = ImpersonatePreset("safari_17.0")
		public val Safari17_2_1: ImpersonatePreset get() = ImpersonatePreset("safari_17.2.1")
		public val Safari17_4_1: ImpersonatePreset get() = ImpersonatePreset("safari_17.4.1")
		public val Safari17_5: ImpersonatePreset get() = ImpersonatePreset("safari_17.5")
		public val Safari18: ImpersonatePreset get() = ImpersonatePreset("safari_18")
		public val SafariIPad18: ImpersonatePreset get() = ImpersonatePreset("safari_ipad_18")

		// OkHttp
		public val OkHttp3_9: ImpersonatePreset get() = ImpersonatePreset("okhttp_3.9")
		public val OkHttp3_11: ImpersonatePreset get() = ImpersonatePreset("okhttp_3.11")
		public val OkHttp3_13: ImpersonatePreset get() = ImpersonatePreset("okhttp_3.13")
		public val OkHttp3_14: ImpersonatePreset get() = ImpersonatePreset("okhttp_3.14")
		public val OkHttp4_9: ImpersonatePreset get() = ImpersonatePreset("okhttp_4.9")
		public val OkHttp4_10: ImpersonatePreset get() = ImpersonatePreset("okhttp_4.10")
		public val OkHttp5: ImpersonatePreset get() = ImpersonatePreset("okhttp_5")

		// Edge
		public val Edge101: ImpersonatePreset get() = ImpersonatePreset("edge_101")
		public val Edge122: ImpersonatePreset get() = ImpersonatePreset("edge_122")
		public val Edge127: ImpersonatePreset get() = ImpersonatePreset("edge_127")
	}
}
