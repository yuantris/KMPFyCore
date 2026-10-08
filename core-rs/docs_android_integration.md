# Android / KMP 集成

假设目录：

```text
workspace/
├── app/
├── settings.gradle.kts
└── core-rs/
```

## settings.gradle.kts

```kotlin
include(":core-rs-android")
project(":core-rs-android").projectDir = file("core-rs/bindings/android")
```

## Android app

```kotlin
dependencies {
    implementation(project(":core-rs-android"))
}
```

## KMP

建议不要让 `commonMain` 直接依赖 Android JNI binding。

推荐：

```text
commonMain
   │
   │ expect
   ▼
CoreSearch / CoreMath / CoreBinary
   ▲
   │ actual
androidMain
   │
   ▼
core-rs-android
```

后续 iOS/JVM/Desktop/HarmonyOS 各自提供 binding。

这样 Rust 核心保持完全平台无关。
