// The JVM binding intentionally reuses the Android JNI implementation.
// Both targets expose the same Java-facing CoreRsNative ABI; only the
// native library name/build target differs.
#[path = "../../android/src/support.rs"]
mod support;
#[path = "../../android/src/math.rs"]
mod math;
#[path = "../../android/src/graph.rs"]
mod graph;
#[path = "../../android/src/search.rs"]
mod search;
#[path = "../../android/src/binary.rs"]
mod binary;
#[path = "../../android/src/calc.rs"]
mod calc;
