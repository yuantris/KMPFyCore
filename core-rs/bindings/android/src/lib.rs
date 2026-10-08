use core_binary::{ApkReader, ZipReader};
use core_graph::{GraphConfig, GraphResult};
use core_math::Expression;
use core_search::SearchEngine;
use jni::objects::{JClass, JString};
use jni::sys::{jboolean, jdouble, jint, jlong, jstring};
use jni::JNIEnv;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, MutexGuard, OnceLock};

static SEARCH_ENGINES: OnceLock<Mutex<HashMap<u64, SearchEngine>>> = OnceLock::new();
static ZIP_READERS: OnceLock<Mutex<HashMap<u64, ZipReader>>> = OnceLock::new();
static NEXT_ID: AtomicU64 = AtomicU64::new(1);

fn searches() -> &'static Mutex<HashMap<u64, SearchEngine>> {
    SEARCH_ENGINES.get_or_init(|| Mutex::new(HashMap::new()))
}

fn zips() -> &'static Mutex<HashMap<u64, ZipReader>> {
    ZIP_READERS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn next_id() -> Option<u64> {
    NEXT_ID.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |value| value.checked_add(1)).ok()
}

fn lock<'a, T>(mutex: &'a Mutex<T>) -> MutexGuard<'a, T> {
    match mutex.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

fn require_handle(handle: jlong, kind: &str) -> Result<u64, String> {
    if handle <= 0 {
        return Err(format!("invalid {kind} handle"));
    }
    Ok(handle as u64)
}

fn string_arg(env: &mut JNIEnv<'_>, value: JString) -> Result<String, ()> {
    env.get_string(&value).map(|value| value.into()).map_err(|error| {
        throw(env, error.to_string());
    })
}

fn throw(env: &mut JNIEnv<'_>, message: impl Into<String>) {
    let _ = env.throw_new("java/lang/RuntimeException", message.into());
}

fn graph_json(result: GraphResult) -> Result<String, serde_json::Error> {
    let payload: Vec<Vec<[f64; 2]>> = result
        .segments
        .into_iter()
        .map(|segment| {
            segment
                .points
                .into_iter()
                .map(|point| [point.x, point.y])
                .collect()
        })
        .collect();
    serde_json::to_string(&payload)
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_core_rs_CoreRsNative_nativeExpressionEval(
    mut env: JNIEnv,
    _class: JClass,
    expression: JString,
) -> jdouble {
    let expression: String = match env.get_string(&expression) {
        Ok(value) => value.into(),
        Err(error) => {
            throw(&mut env, error.to_string());
            return f64::NAN;
        }
    };

    match Expression::eval(&expression) {
        Ok(value) => value,
        Err(error) => {
            throw(&mut env, error.to_string());
            f64::NAN
        }
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_core_rs_CoreRsNative_nativeExpressionEvalX(
    mut env: JNIEnv, _class: JClass, expression: JString, x: jdouble,
) -> jdouble {
    let Ok(expression) = string_arg(&mut env, expression) else { return f64::NAN; };
    match Expression::eval_x(&expression, x) {
        Ok(value) => value,
        Err(error) => {
            throw(&mut env, error.to_string());
            f64::NAN
        }
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_core_rs_CoreRsNative_nativeExpressionContainsVariable(
    mut env: JNIEnv, _class: JClass, expression: JString,
) -> jboolean {
    let Ok(expression) = string_arg(&mut env, expression) else { return 0; };
    match Expression::compile(&expression) {
        Ok(expr) => expr.contains_variable() as jboolean,
        Err(error) => {
            throw(&mut env, error.to_string());
            0
        }
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_core_rs_CoreRsNative_nativeGraph(
    mut env: JNIEnv,
    _class: JClass,
    expression: JString,
    min_x: jdouble,
    max_x: jdouble,
    min_y: jdouble,
    max_y: jdouble,
    samples: jint,
    pixel_width: jint,
    pixel_height: jint,
) -> jstring {
    let expression: String = match env.get_string(&expression) {
        Ok(value) => value.into(),
        Err(error) => {
            throw(&mut env, error.to_string());
            return std::ptr::null_mut();
        }
    };

    let expr = match Expression::compile(&expression) {
        Ok(expr) => expr,
        Err(error) => {
            throw(&mut env, error.to_string());
            return std::ptr::null_mut();
        }
    };

    let config = GraphConfig {
        min_x,
        max_x,
        min_y,
        max_y,
        samples: samples.max(64) as usize,
        pixel_width: pixel_width.max(2) as u32,
        pixel_height: pixel_height.max(2) as u32,
        ..GraphConfig::default()
    };

    let result = match core_graph::sample_checked(&expr, &config) {
        Ok(result) => result,
        Err(error) => {
            throw(&mut env, error.to_string());
            return std::ptr::null_mut();
        }
    };

    match graph_json(result) {
        Ok(json) => match env.new_string(json) {
            Ok(value) => value.into_raw(),
            Err(error) => {
                throw(&mut env, error.to_string());
                std::ptr::null_mut()
            }
        },
        Err(error) => {
            throw(&mut env, error.to_string());
            std::ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_core_rs_CoreRsNative_nativeSearchCreate(
    _env: JNIEnv,
    _class: JClass,
) -> jlong {
    let Some(id) = next_id() else {
        throw(&mut env, "native handle space exhausted");
        return 0;
    };
    lock(searches()).insert(id, SearchEngine::new());
    id as jlong
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_core_rs_CoreRsNative_nativeSearchDestroy(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) {
    let Ok(handle) = require_handle(handle, "search") else {
        throw(&mut env, "invalid search handle");
        return;
    };
    lock(searches()).remove(&handle);
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_core_rs_CoreRsNative_nativeSearchAdd(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
    id: jlong,
    text: JString,
) -> jboolean {
    let Ok(handle) = require_handle(handle, "search") else {
        throw(&mut env, "invalid search handle");
        return 0;
    };
    let Ok(id) = u64::try_from(id) else {
        throw(&mut env, "id must be >= 0");
        return 0;
    };
    let Ok(text) = string_arg(&mut env, text) else { return 0; };

    let mut engines = lock(searches());
    let Some(engine) = engines.get_mut(&handle) else {
        throw(&mut env, "invalid search engine handle");
        return 0;
    };

    match engine.add(id, &text) {
        Ok(()) => 1,
        Err(error) => {
            throw(&mut env, error.to_string());
            0
        }
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_core_rs_CoreRsNative_nativeSearchRemove(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
    id: jlong,
) -> jboolean {
    let Ok(handle) = require_handle(handle, "search") else {
        throw(&mut env, "invalid search handle");
        return 0;
    };
    let Ok(id) = u64::try_from(id) else {
        throw(&mut env, "id must be >= 0");
        return 0;
    };
    let mut engines = lock(searches());
    let Some(engine) = engines.get_mut(&handle) else {
        throw(&mut env, "invalid search handle");
        return 0;
    };
    engine.remove(id) as jboolean
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_core_rs_CoreRsNative_nativeSearchClear(
    mut env: JNIEnv, _class: JClass, handle: jlong,
) {
    let Ok(handle) = require_handle(handle, "search") else {
        throw(&mut env, "invalid search handle"); return;
    };
    let mut engines = lock(searches());
    let Some(engine) = engines.get_mut(&handle) else {
        throw(&mut env, "invalid search handle"); return;
    };
    engine.clear();
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_core_rs_CoreRsNative_nativeSearchSize(
    mut env: JNIEnv, _class: JClass, handle: jlong,
) -> jint {
    let Ok(handle) = require_handle(handle, "search") else {
        throw(&mut env, "invalid search handle"); return 0;
    };
    let engines = lock(searches());
    let Some(engine) = engines.get(&handle) else {
        throw(&mut env, "invalid search handle"); return 0;
    };
    engine.len().min(jint::MAX as usize) as jint
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_core_rs_CoreRsNative_nativeSearch(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
    query: JString,
    limit: jint,
) -> jstring {
    let Ok(handle) = require_handle(handle, "search") else {
        throw(&mut env, "invalid search handle");
        return std::ptr::null_mut();
    };
    let Ok(query) = string_arg(&mut env, query) else { return std::ptr::null_mut(); };
    if limit < 0 {
        throw(&mut env, "limit must be >= 0");
        return std::ptr::null_mut();
    }

    let engines = lock(searches());
    let Some(engine) = engines.get(&handle) else {
        throw(&mut env, "invalid search engine handle");
        return std::ptr::null_mut();
    };

    let json = match serde_json::to_string(&engine.search(&query, limit.max(0) as usize)) {
        Ok(value) => value,
        Err(error) => {
            throw(&mut env, error.to_string());
            return std::ptr::null_mut();
        }
    };

    match env.new_string(json) {
        Ok(value) => value.into_raw(),
        Err(error) => {
            throw(&mut env, error.to_string());
            std::ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_core_rs_CoreRsNative_nativeZipCreate(
    mut env: JNIEnv,
    _class: JClass,
    path: JString,
) -> jlong {
    let path: String = match env.get_string(&path) {
        Ok(value) => value.into(),
        Err(error) => {
            throw(&mut env, error.to_string());
            return 0;
        }
    };

    match ZipReader::open(path) {
        Ok(reader) => {
            let Some(id) = next_id() else {
                throw(&mut env, "native handle space exhausted");
                return 0;
            };
            lock(zips()).insert(id, reader);
            id as jlong
        }
        Err(error) => {
            throw(&mut env, error.to_string());
            0
        }
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_core_rs_CoreRsNative_nativeZipDestroy(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) {
    let Ok(handle) = require_handle(handle, "zip") else {
        throw(&mut env, "invalid zip handle");
        return;
    };
    lock(zips()).remove(&handle);
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_core_rs_CoreRsNative_nativeZipEntries(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
) -> jstring {
    let Ok(handle) = require_handle(handle, "zip") else {
        throw(&mut env, "invalid zip handle");
        return std::ptr::null_mut();
    };
    let readers = lock(zips());
    let Some(reader) = readers.get(&handle) else {
        throw(&mut env, "invalid zip handle");
        return std::ptr::null_mut();
    };

    let entries = match reader.entries() {
        Ok(entries) => entries,
        Err(error) => {
            throw(&mut env, error.to_string());
            return std::ptr::null_mut();
        }
    };

    let json = match serde_json::to_string(&entries) {
        Ok(value) => value,
        Err(error) => {
            throw(&mut env, error.to_string());
            return std::ptr::null_mut();
        }
    };

    match env.new_string(json) {
        Ok(value) => value.into_raw(),
        Err(error) => {
            throw(&mut env, error.to_string());
            std::ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_core_rs_CoreRsNative_nativeZipContains(
    mut env: JNIEnv, _class: JClass, handle: jlong, name: JString,
) -> jboolean {
    let Ok(handle) = require_handle(handle, "zip") else {
        throw(&mut env, "invalid zip handle"); return 0;
    };
    let Ok(name) = string_arg(&mut env, name) else { return 0; };
    let readers = lock(zips());
    let Some(reader) = readers.get(&handle) else {
        throw(&mut env, "invalid zip handle"); return 0;
    };
    match reader.contains(&name) {
        Ok(value) => value as jboolean,
        Err(error) => {
            throw(&mut env, error.to_string());
            0
        }
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_core_rs_CoreRsNative_nativeApkIsApk(
    mut env: JNIEnv,
    _class: JClass,
    path: JString,
) -> jboolean {
    let path: String = match env.get_string(&path) {
        Ok(value) => value.into(),
        Err(error) => {
            throw(&mut env, error.to_string());
            return 0;
        }
    };

    match ApkReader::open(path).and_then(|apk| apk.is_apk()) {
        Ok(value) => value as jboolean,
        Err(error) => {
            throw(&mut env, error.to_string());
            0
        }
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_core_rs_CoreRsNative_nativeApkEntries(
    mut env: JNIEnv, _class: JClass, path: JString,
) -> jstring {
    let Ok(path) = string_arg(&mut env, path) else { return std::ptr::null_mut(); };
    let apk = match ApkReader::open(path) {
        Ok(value) => value,
        Err(error) => {
            throw(&mut env, error.to_string());
            return std::ptr::null_mut();
        }
    };
    let entries = match apk.entries() {
        Ok(value) => value,
        Err(error) => {
            throw(&mut env, error.to_string());
            return std::ptr::null_mut();
        }
    };
    let json = match serde_json::to_string(&entries) {
        Ok(value) => value,
        Err(error) => {
            throw(&mut env, error.to_string());
            return std::ptr::null_mut();
        }
    };
    match env.new_string(json) {
        Ok(value) => value.into_raw(),
        Err(error) => {
            throw(&mut env, error.to_string());
            std::ptr::null_mut()
        }
    }
}
