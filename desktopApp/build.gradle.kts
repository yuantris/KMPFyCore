import org.jetbrains.compose.desktop.application.dsl.TargetFormat
import org.gradle.api.tasks.Exec

plugins {
    alias(libs.plugins.kotlinJvm)
    alias(libs.plugins.composeMultiplatform)
    alias(libs.plugins.composeCompiler)
}

dependencies {
    implementation(project(":shared"))
    implementation(compose.desktop.currentOs)
    implementation(libs.kotlinx.coroutinesSwing)
    implementation(libs.compose.uiToolingPreview)
}

val coreRsResourcesDir = layout.buildDirectory.dir("core-rs-resources")

fun currentOsDir(): String = when {
    System.getProperty("os.name").contains("Windows", true) -> "windows"
    System.getProperty("os.name").contains("Mac", true) -> "macos"
    else -> "linux"
}

fun currentArchDir(): String = when (System.getProperty("os.arch").lowercase()) {
    "aarch64", "arm64" -> "arm64"
    else -> "x64"
}

fun nativeFileName(): String = when (currentOsDir()) {
    "windows" -> "core_rs_jvm.dll"
    "macos" -> "libcore_rs_jvm.dylib"
    else -> "libcore_rs_jvm.so"
}

tasks.register<Exec>("buildCoreRsJvm") {
    workingDir(rootProject.projectDir)
    commandLine("cargo", "build", "--manifest-path", "core-rs/bindings/jvm/Cargo.toml")
    doLast {
        val fileName = nativeFileName()
        val source = rootProject.file("core-rs/target/debug/$fileName")
        check(source.isFile) { "Cargo did not produce $source" }

        val destination = coreRsResourcesDir.get().asFile
            .resolve(currentOsDir() + "-" + currentArchDir())
            .resolve(fileName)

        destination.parentFile.mkdirs()
        source.copyTo(destination, overwrite = true)
    }
}

compose.desktop {
    application {
        mainClass = "com.core.fy.kmp.MainKt"
        dependsOn("buildCoreRsJvm")

        nativeDistributions {
            targetFormats(
                TargetFormat.Dmg,
                TargetFormat.Msi,
                TargetFormat.Exe,
                TargetFormat.Deb,
                TargetFormat.Rpm,
            )
            packageName = "com.core.fy.kmp"
            packageVersion = "1.0.0"
            appResourcesRootDir.set(coreRsResourcesDir)
        }
    }
}
