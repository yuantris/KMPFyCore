use core_search::SearchEngine;
use jni::objects::{JClass, JString};
use jni::sys::{jboolean, jlong, jint, jstring};
use jni::JNIEnv;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use crate::support::{lock, next_id, require_handle, string_arg, throw};

static SEARCH_ENGINES: OnceLock<Mutex<HashMap<u64, SearchEngine>>> = OnceLock::new();

fn searches() -> &'static Mutex<HashMap<u64, SearchEngine>> {
    SEARCH_ENGINES.get_or_init(|| Mutex::new(HashMap::new()))
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_core_rs_CoreRsNative_nativeSearchCreate(
    mut env: JNIEnv,
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
    mut env: JNIEnv,
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
