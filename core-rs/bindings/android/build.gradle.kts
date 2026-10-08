plugins {
    id("com.android.library")
}

// ---- Rust (cargo-ndk) 一键编译 ----
// 运行 `./gradlew :androidApp:assembleDebug` 会自动先执行本任务编译原生库；
// 也可单独执行 `./gradlew :core-rs-android:buildRustAndroid`。
val rustWorkspaceDir = project.projectDir.resolve("../../")
val jniLibsDir = project.projectDir.resolve("src/main/jniLibs")

// 定位 Android NDK：优先 gradle.properties 的 android.ndk.dir，其次环境变量，
// 最后从常见 SDK 目录自动探测（优先 27 系列，符合项目已验证的构建环境）。
fun findAndroidNdk(): String {
    val fromProperty = providers.gradleProperty("android.ndk.dir").orNull
    if (!fromProperty.isNullOrBlank() && File(fromProperty).isDirectory) return fromProperty

    for (name in listOf("ANDROID_NDK_HOME", "ANDROID_NDK_ROOT")) {
        val value = System.getenv(name)
        if (!value.isNullOrBlank() && File(value).isDirectory) return value
    }

    val sdkRoots = mutableListOf<String>()
    for (name in listOf("ANDROID_SDK_ROOT", "ANDROID_HOME")) {
        System.getenv(name)?.let { sdkRoots += it }
    }
    val userHome = System.getProperty("user.home")
    sdkRoots += listOf(
        "D:/Android/Sdk",
        "C:/Android/Sdk",
        "$userHome/AppData/Local/Android/Sdk",
        "$userHome/Android/Sdk"
    )

    val candidates = sdkRoots.asSequence()
        .map { File(it, "ndk") }
        .filter { it.isDirectory }
        .flatMap { it.listFiles { f -> f.isDirectory }?.asSequence() ?: emptySequence() }
        .toList()
    val chosen = candidates.filter { it.name.startsWith("27") }
        .maxByOrNull { it.name }
        ?: candidates.maxByOrNull { it.name }
    if (chosen != null) return chosen.absolutePath

    throw GradleException(
        "找不到 Android NDK。请设置环境变量 ANDROID_NDK_HOME，或在 gradle.properties 中添加 android.ndk.dir=<ndk路径>"
    )
}

val buildRustAndroid by tasks.registering(Exec::class) {
    group = "rust"
    description = "使用 cargo-ndk 编译 core-rs 的 Android 原生库并输出到 jniLibs"
    workingDir = rustWorkspaceDir
    val ndk = findAndroidNdk()
    environment("ANDROID_NDK_HOME", ndk)
    System.getenv("ANDROID_SDK_ROOT")?.let { environment("ANDROID_SDK_ROOT", it) }
    commandLine(
        "cargo", "ndk",
        "-t", "arm64-v8a",
        "-t", "armeabi-v7a",
        "-t", "x86_64",
        "-o", jniLibsDir.absolutePath,
        "build", "-p", "core-rs-android", "--release"
    )
    doFirst { logger.lifecycle("cargo-ndk 使用的 NDK: $ndk") }
}

// 让 Android 构建（含 androidApp）自动先编译 Rust 原生库
tasks.named("preBuild") { dependsOn(buildRustAndroid) }

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
