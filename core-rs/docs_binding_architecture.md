# Android Binding Architecture

The Android binding is a thin FFI adapter. It owns JNI handles and JSON/string marshaling, but it does not contain business logic.

## Layout

```text
core-rs/
├── crates/
│   ├── core-common/     # shared error/result
│   ├── core-math/       # lexer/parser/AST/evaluation
│   ├── core-graph/      # adaptive graph sampling
│   ├── core-search/     # search index
│   └── core-binary/     # ZIP/APK readers
└── bindings/android/
    └── src/
        ├── lib.rs       # module entry only
        ├── support.rs   # handle allocator, mutex access, JNI args/errors
        ├── math.rs      # math JNI exports
        ├── graph.rs     # graph JNI export
        ├── search.rs    # SearchEngine JNI exports
        └── binary.rs    # ZipReader / ApkReader JNI exports
```

## FFI rules

1. Never allow a Rust panic to cross JNI.
2. JNI methods return immediately after setting a Java exception.
3. 0 is reserved as an invalid opaque handle.
4. Negative Java Long handles are rejected.
5. Handle allocation is monotonic and overflow-safe.
6. Destroy operations are safe to call more than once from the Kotlin wrapper.
7. Core crates must not depend on Android, JNI, Compose, or Kotlin.
8. JSON is used only at the binding boundary for variable-size result collections.
9. Graph sampling compiles the expression once and evaluates the AST repeatedly.
10. Resource-heavy graph requests are bounded before sampling.

## Kotlin ownership

Long-lived native resources implement AutoCloseable:

- SearchEngine
- ZipReader

The Kotlin object owns the opaque handle and calls the corresponding native destroy function from close().

Stateless operations such as Expression, CoreGraph, and ApkReader do not expose native handles.

## Native API surface

### Math

- constant expression evaluation
- f(x) evaluation
- variable detection

### Graph

- adaptive curve sampling
- discontinuity/domain splitting
- screen-space error control

### Search

- create/destroy
- add/remove
- clear
- size
- ranked search

### Binary

- ZIP create/destroy
- ZIP entries
- ZIP contains
- APK detection
- APK entries

## Build

Use:

```bash
./scripts/test.sh
```

for formatting, workspace tests and clippy.

Use:

```bash
./scripts/build-android.sh
```

to regenerate the Android .so files after changing JNI or Rust core code.
