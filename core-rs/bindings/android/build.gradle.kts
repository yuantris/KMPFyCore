plugins {
    id("com.android.library")
    kotlin("android")
    kotlin("plugin.serialization")
}

android {
    namespace = "io.corer.rs"
    compileSdk = 36
    defaultConfig { minSdk = 24 }

    sourceSets["main"].jniLibs.srcDir("src/main/jniLibs")

    packaging {
        jniLibs { useLegacyPackaging = false }
    }
}

dependencies {
    implementation("org.jetbrains.kotlinx:kotlinx-serialization-json:1.9.0")
}
