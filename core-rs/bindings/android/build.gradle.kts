plugins {
    id("com.android.library")
    alias(libs.plugins.kotlinSerialization)
}

android {
    namespace = "io.core.rs"
    compileSdk = libs.versions.android.compileSdk.get().toInt()
    defaultConfig { minSdk = libs.versions.android.minSdk.get().toInt() }

    sourceSets["main"].jniLibs.srcDir("src/main/jniLibs")

    packaging {
        jniLibs { useLegacyPackaging = false }
    }
}

dependencies {
    implementation("org.jetbrains.kotlinx:kotlinx-serialization-json:1.9.0")
}
