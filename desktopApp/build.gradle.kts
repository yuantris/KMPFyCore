import org.jetbrains.compose.desktop.application.dsl.TargetFormat
import org.gradle.internal.os.OperatingSystem

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

tasks.register<Exec>("buildCoreRsJvm") {
    workingDir(rootProject.projectDir)
    commandLine("cargo", "build", "--manifest-path", "core-rs/bindings/jvm/Cargo.toml")
    doLast {
        val os = OperatingSystem.current()
        val fileName = when {
            os.isWindows -> "core_rs_jvm.dll"
            os.isMacOsX -> "libcore_rs_jvm.dylib"
            else -> "libcore_rs_jvm.so"
        }
        val source = rootProject.file("core-rs/target/debug/$fileName")
        val destination = coreRsJvmDir.get().asFile.resolve(fileName)
        destination.parentFile.mkdirs()
        source.copyTo(destination, overwrite = true)
    }
}

compose.desktop {
    application {
        mainClass = "com.core.fy.kmp.MainKt"
        dependsOn("buildCoreRsJvm")
        jvmArgs("-Dcore.rs.native.path=${coreRsJvmDir.get().asFile.resolve("core_rs_jvm" + when {
            OperatingSystem.current().isWindows -> ".dll"
            OperatingSystem.current().isMacOsX -> ".dylib"
            else -> ".so"
        })}")
        nativeDistributions {
            targetFormats(TargetFormat.Dmg, TargetFormat.Msi, TargetFormat.Deb)
            packageName = "com.core.fy.kmp"
            packageVersion = "1.0.0"
        }
    }
}
