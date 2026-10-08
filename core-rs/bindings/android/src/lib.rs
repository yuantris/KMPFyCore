use core_binary::{ApkReader, ZipReader};
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

fn searches() -> &'static Mutex<HashMap<u64, SearchEngine>> { SEARCH_ENGINES.get_or_init(|| Mutex::new(HashMap::new())) }
fn zips() -> &'static Mutex<HashMap<u64, ZipReader>> { ZIP_READERS.get_or_init(|| Mutex::new(HashMap::new())) }
fn next_id() -> u64 {
    let lock = NEXT_ID.get_or_init(|| Mutex::new(1));
    let mut value = lock.lock().expect("id mutex poisoned");
    let result = *value;
    *value += 1;
    result
}
fn throw(env: &mut JNIEnv<'_>, message: impl Into<String>) { let _ = env.throw_new("java/lang/RuntimeException", message.into()); }

#[no_mangle]
pub extern "system" fn Java_io_corer_rs_CoreRsNative_nativeExpressionEval(mut env: JNIEnv, _class: JClass, expression: JString) -> jdouble {
    let expression: String = match env.get_string(&expression) { Ok(v) => v.into(), Err(e) => { throw(&mut env, e.to_string()); return f64::NAN; } };
    match Expression::eval(&expression) { Ok(v) => v, Err(e) => { throw(&mut env, e.to_string()); f64::NAN } }
}

#[no_mangle]
pub extern "system" fn Java_io_corer_rs_CoreRsNative_nativeSearchCreate(_env: JNIEnv, _class: JClass) -> jlong {
    let id = next_id();
    searches().lock().expect("search mutex poisoned").insert(id, SearchEngine::new());
    id as jlong
}

#[no_mangle]
pub extern "system" fn Java_io_corer_rs_CoreRsNative_nativeSearchDestroy(_env: JNIEnv, _class: JClass, handle: jlong) {
    searches().lock().expect("search mutex poisoned").remove(&(handle as u64));
}

#[no_mangle]
pub extern "system" fn Java_io_corer_rs_CoreRsNative_nativeSearchAdd(mut env: JNIEnv, _class: JClass, handle: jlong, id: jlong, text: JString) -> jboolean {
    let text: String = match env.get_string(&text) { Ok(v) => v.into(), Err(e) => { throw(&mut env, e.to_string()); return 0; } };
    let mut engines = searches().lock().expect("search mutex poisoned");
    let Some(engine) = engines.get_mut(&(handle as u64)) else { throw(&mut env, "invalid search engine handle"); return 0; };
    match engine.add(id as u64, &text) { Ok(()) => 1, Err(e) => { throw(&mut env, e.to_string()); 0 } }
}

#[no_mangle]
pub extern "system" fn Java_io_corer_rs_CoreRsNative_nativeSearchRemove(_env: JNIEnv, _class: JClass, handle: jlong, id: jlong) -> jboolean {
    let mut engines = searches().lock().expect("search mutex poisoned");
    engines.get_mut(&(handle as u64)).map(|e| e.remove(id as u64)).unwrap_or(false) as jboolean
}

#[no_mangle]
pub extern "system" fn Java_io_corer_rs_CoreRsNative_nativeSearch(mut env: JNIEnv, _class: JClass, handle: jlong, query: JString, limit: jint) -> jstring {
    let query: String = match env.get_string(&query) { Ok(v) => v.into(), Err(e) => { throw(&mut env, e.to_string()); return std::ptr::null_mut(); } };
    let engines = searches().lock().expect("search mutex poisoned");
    let Some(engine) = engines.get(&(handle as u64)) else { throw(&mut env, "invalid search engine handle"); return std::ptr::null_mut(); };
    let results = engine.search(&query, limit.max(0) as usize);
    let json = match serde_json::to_string(&results) { Ok(v) => v, Err(e) => { throw(&mut env, e.to_string()); return std::ptr::null_mut(); } };
    match env.new_string(json) { Ok(v) => v.into_raw(), Err(e) => { throw(&mut env, e.to_string()); std::ptr::null_mut() } }
}

#[no_mangle]
pub extern "system" fn Java_io_corer_rs_CoreRsNative_nativeZipCreate(mut env: JNIEnv, _class: JClass, path: JString) -> jlong {
    let path: String = match env.get_string(&path) { Ok(v) => v.into(), Err(e) => { throw(&mut env, e.to_string()); return 0; } };
    match ZipReader::open(path) {
        Ok(reader) => { let id = next_id(); zips().lock().expect("zip mutex poisoned").insert(id, reader); id as jlong }
        Err(e) => { throw(&mut env, e.to_string()); 0 }
    }
}

#[no_mangle]
pub extern "system" fn Java_io_corer_rs_CoreRsNative_nativeZipDestroy(_env: JNIEnv, _class: JClass, handle: jlong) {
    zips().lock().expect("zip mutex poisoned").remove(&(handle as u64));
}

#[no_mangle]
pub extern "system" fn Java_io_corer_rs_CoreRsNative_nativeZipEntries(mut env: JNIEnv, _class: JClass, handle: jlong) -> jstring {
    let zips = zips().lock().expect("zip mutex poisoned");
    let Some(reader) = zips.get(&(handle as u64)) else { throw(&mut env, "invalid zip handle"); return std::ptr::null_mut(); };
    let entries = match reader.entries() { Ok(v) => v, Err(e) => { throw(&mut env, e.to_string()); return std::ptr::null_mut(); } };
    let json = match serde_json::to_string(&entries) { Ok(v) => v, Err(e) => { throw(&mut env, e.to_string()); return std::ptr::null_mut(); } };
    match env.new_string(json) { Ok(v) => v.into_raw(), Err(e) => { throw(&mut env, e.to_string()); std::ptr::null_mut() } }
}

#[no_mangle]
pub extern "system" fn Java_io_corer_rs_CoreRsNative_nativeApkIsApk(mut env: JNIEnv, _class: JClass, path: JString) -> jboolean {
    let path: String = match env.get_string(&path) { Ok(v) => v.into(), Err(e) => { throw(&mut env, e.to_string()); return 0; } };
    match ApkReader::open(path).and_then(|apk| apk.is_apk()) { Ok(v) => v as jboolean, Err(e) => { throw(&mut env, e.to_string()); 0 } }
}
