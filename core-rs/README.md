# core-rs

跨平台高性能基础能力库。

## 设计目标

- Rust 负责 CPU 密集型、数据密集型逻辑
- Kotlin/KMP/CMP 负责业务与 UI
- Android 通过 JNI 接入
- 后续可增加 iOS/Swift、Desktop/JVM、HarmonyOS bindings
- 核心 crate 不依赖任何平台 UI/API

## 当前模块

### core-common

公共 `Result/Error` 类型。

### core-search

第一版提供：

- Prefix Search
- Contains Search
- 简单 Fuzzy Search
- Top-K 限制

### core-math

第一版提供：

- `+ - * / % ^`
- 括号
- 一元正负号
- `sin cos tan`
- `sqrt`
- `abs`
- `ln`
- `log`
- `pi`
- `e`

三角函数默认使用弧度。

### core-graph

独立于 `core-math` 的函数曲线采样层：

- 自适应 screen-space 细分
- finite/domain 检测
- 渐近线/不连续区间自动断开
- 资源上限保护
- `sample_checked` 返回可传递的错误

### core-binary

ZIP/APK 基础读取：

- ZIP entry 列表
- `contains(name)`
- APK 基础检查
- APK entry 列表

Manifest/AXML/DEX 解析留给后续版本。

### bindings/android

JNI 按职责拆分为：

```text
src/
├── lib.rs       # 模块入口
├── support.rs   # handle / Mutex / JNI 参数与异常
├── math.rs      # Expression
├── graph.rs     # Graph
├── search.rs    # SearchEngine
└── binary.rs    # ZipReader / ApkReader
```

通过 JNI 暴露：

- Expression：常量求值、`f(x)`、变量检测
- Graph：函数曲线采样
- SearchEngine：add/remove/clear/size/search
- ZipReader：entries/contains
- ApkReader：isApk/entries

## 原则

1. Core 不知道 Android / Compose / Swift / ArkUI。
2. 一次 FFI 调用尽量处理一批数据。
3. 避免逐元素跨语言调用。
4. 性能敏感模块必须有 benchmark。
5. FFI API 与内部实现解耦。

## Rust

```bash
cargo test --workspace
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

## Android

安装：

```bash
cargo install cargo-ndk
```

然后：

```bash
./scripts/build-android.sh
```

动态库会放到：

```text
bindings/android/src/main/jniLibs/
├── arm64-v8a/libcore_rs_android.so
├── armeabi-v7a/libcore_rs_android.so
└── x86_64/libcore_rs_android.so
```

Android binding 是一个可被宿主 Android/KMP 工程直接纳入的 library module。把 `bindings/android` 作为 module directory 指向即可。

如果宿主工程已经有 `settings.gradle.kts`，可以使用：

```kotlin
include(":core-rs-android")
project(":core-rs-android").projectDir = file("../core-rs/bindings/android")
```

然后：

```kotlin
dependencies {
    implementation(project(":core-rs-android"))
}
```
