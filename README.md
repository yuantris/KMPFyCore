# FyCoreKmp

Kotlin Multiplatform + Compose Multiplatform application shell for FyCore.

## Modules

- `shared` — common Compose UI and shared Kotlin code.
- `androidApp` — Android application entry point.
- `desktopApp` — Desktop/JVM application entry point.
- `core-rs` — Rust high-performance core, kept platform-independent and exposed to Android through a dedicated JNI binding.

## Run

- `./gradlew :androidApp:assembleDebug`
- `./gradlew :desktopApp:run`

## Test

- `./gradlew :shared:jvmTest`
- `cd core-rs && cargo test --workspace`

The application UI does not depend directly on Android APIs. Platform-specific integrations belong in the corresponding source set or binding module.
