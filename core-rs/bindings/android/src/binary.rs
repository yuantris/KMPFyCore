use core_binary::{ApkReader, ZipReader};
use jni::objects::{JClass, JString};
use jni::sys::{jboolean, jlong, jstring};
use jni::JNIEnv;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use crate::support::{lock, next_id, require_handle, string_arg, throw};

static ZIP_READERS: OnceLock<Mutex<HashMap<u64, ZipReader>>> = OnceLock::new();

fn zips() -> &'static Mutex<HashMap<u64, ZipReader>> {
    ZIP_READERS.get_or_init(|| Mutex::new(HashMap::new()))
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