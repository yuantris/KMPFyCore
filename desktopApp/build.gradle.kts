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

val coreRsJvmDir = layout.buildDirectory.dir("core-rs-jvm")
val coreRsJvmLibraryName = when {
    System.getProperty("os.name").contains("Windows", ignoreCase = true) -> "core_rs_jvm.dll"
    System.getProperty("os.name").contains("Mac", ignoreCase = true) -> "libcore_rs_jvm.dylib"
    else -> "libcore_rs_jvm.so"
}

tasks.register<Exec>("buildCoreRsJvm") {
    workingDir(rootProject.projectDir)
    commandLine("cargo", "build", "--manifest-path", "core-rs/bindings/jvm/Cargo.toml")
    doLast {
        val source = rootProject.file("core-rs/target/debug/$coreRsJvmLibraryName")
        val destination = coreRsJvmDir.get().asFile.resolve(coreRsJvmLibraryName)
        check(source.isFile) { "Cargo did not produce $source" }
        destination.parentFile.mkdirs()
        source.copyTo(destination, overwrite = true)
    }
}

compose.desktop {
    application {
        mainClass = "com.core.fy.kmp.MainKt"
        dependsOn("buildCoreRsJvm")
        jvmArgs("-Dcore.rs.native.path=${coreRsJvmDir.get().asFile.resolve(coreRsJvmLibraryName)}")

        nativeDistributions {
            targetFormats(TargetFormat.Dmg, TargetFormat.Msi, TargetFormat.Deb)
            packageName = "com.core.fy.kmp"
            packageVersion = "1.0.0"
        }
    }
}
