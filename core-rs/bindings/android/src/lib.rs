use core_binary::{ApkReader, ZipReader};
use core_graph::{GraphConfig, GraphResult};
use core_math::Expression;
use core_search::SearchEngine;
use jni::objects::{JClass, JString};
use jni::sys::{jboolean, jdouble, jint, jlong, jstring};
use jni::JNIEnv;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

static SEARCH_ENGINES: OnceLock<Mutex<HashMap<u64, SearchEngine>>> = OnceLock::new();
static ZIP_READERS: OnceLock<Mutex<HashMap<u64, ZipReader>>> = OnceLock::new();
static NEXT_ID: OnceLock<Mutex<u64>> = OnceLock::new();

fn searches() -> &'static Mutex<HashMap<u64, SearchEngine>> {
    SEARCH_ENGINES.get_or_init(|| Mutex::new(HashMap::new()))
}

fn zips() -> &'static Mutex<HashMap<u64, ZipReader>> {
    ZIP_READERS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn next_id() -> u64 {
    let lock = NEXT_ID.get_or_init(|| Mutex::new(1));
    let mut value = lock.lock().expect("id mutex poisoned");
    let result = *value;
    *value += 1;
    result
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

    let result = core_graph::sample(&expr, &config);

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
    let id = next_id();
    searches()
        .lock()
        .expect("search mutex poisoned")
        .insert(id, SearchEngine::new());
    id as jlong
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_core_rs_CoreRsNative_nativeSearchDestroy(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) {
    searches()
        .lock()
        .expect("search mutex poisoned")
        .remove(&(handle as u64));
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_core_rs_CoreRsNative_nativeSearchAdd(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
    id: jlong,
    text: JString,
) -> jboolean {
    let text: String = match env.get_string(&text) {
        Ok(value) => value.into(),
        Err(error) => {
            throw(&mut env, error.to_string());
            return 0;
        }
    };

    let mut engines = searches().lock().expect("search mutex poisoned");
    let Some(engine) = engines.get_mut(&(handle as u64)) else {
        throw(&mut env, "invalid search engine handle");
        return 0;
    };

    match engine.add(id as u64, &text) {
        Ok(()) => 1,
        Err(error) => {
            throw(&mut env, error.to_string());
            0
        }
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_core_rs_CoreRsNative_nativeSearchRemove(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
    id: jlong,
) -> jboolean {
    searches()
        .lock()
        .expect("search mutex poisoned")
        .get_mut(&(handle as u64))
        .map(|engine| engine.remove(id as u64))
        .unwrap_or(false) as jboolean
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_core_rs_CoreRsNative_nativeSearch(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
    query: JString,
    limit: jint,
) -> jstring {
    let query: String = match env.get_string(&query) {
        Ok(value) => value.into(),
        Err(error) => {
            throw(&mut env, error.to_string());
            return std::ptr::null_mut();
        }
    };

    let engines = searches().lock().expect("search mutex poisoned");
    let Some(engine) = engines.get(&(handle as u64)) else {
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
            let id = next_id();
            zips()
                .lock()
                .expect("zip mutex poisoned")
                .insert(id, reader);
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
    zips()
        .lock()
        .expect("zip mutex poisoned")
        .remove(&(handle as u64));
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_core_rs_CoreRsNative_nativeZipEntries(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
) -> jstring {
    let readers = zips().lock().expect("zip mutex poisoned");
    let Some(reader) = readers.get(&(handle as u64)) else {
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
