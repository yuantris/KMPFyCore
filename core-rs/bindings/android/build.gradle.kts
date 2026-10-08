plugins {
    id("com.android.library")
}

android {
    namespace = "io.core.rs"
    compileSdk = libs.versions.android.compileSdk.get().toInt()
    defaultConfig {
        minSdk = libs.versions.android.minSdk.get().toInt()
    }

    sourceSets["main"].jniLibs.srcDir("src/main/jniLibs")

    packaging {
        jniLibs {
            useLegacyPackaging = false
        }
    }
}
