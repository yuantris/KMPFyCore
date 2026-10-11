use crate::support::{string_arg, throw};
use core_stitch::{stitch_files, StitchConfig};
use jni::objects::{JClass, JString};
use jni::sys::jstring;
use jni::JNIEnv;
use std::path::PathBuf;

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_core_rs_CoreRsNative_nativeStitch(
    mut env: JNIEnv,
    _class: JClass,
    input_paths_json: JString,
    output_path: JString,
    config_json: JString,
) -> jstring {
    let inputs = match string_arg(&mut env, input_paths_json) {
        Ok(value) => value,
        Err(()) => return std::ptr::null_mut(),
    };
    let output = match string_arg(&mut env, output_path) {
        Ok(value) => value,
        Err(()) => return std::ptr::null_mut(),
    };
    let config_text = match string_arg(&mut env, config_json) {
        Ok(value) => value,
        Err(()) => return std::ptr::null_mut(),
    };

    let input_paths: Vec<String> = match serde_json::from_str::<Vec<String>>(&inputs) {
        Ok(value) if !value.is_empty() => value,
        Ok(_) => {
            throw(&mut env, "at least one input image is required");
            return std::ptr::null_mut();
        }
        Err(error) => {
            throw(&mut env, format!("invalid input path JSON: {error}"));
            return std::ptr::null_mut();
        }
    };
    let config = if config_text.trim().is_empty() {
        StitchConfig::default()
    } else {
        match serde_json::from_str::<StitchConfig>(&config_text) {
            Ok(value) => value,
            Err(error) => {
                throw(&mut env, format!("invalid stitch config JSON: {error}"));
                return std::ptr::null_mut();
            }
        }
    };

    let paths: Vec<PathBuf> = input_paths.into_iter().map(PathBuf::from).collect();
    let result = match stitch_files(&paths, std::path::Path::new(&output), &config) {
        Ok(value) => value,
        Err(error) => {
            throw(&mut env, error.to_string());
            return std::ptr::null_mut();
        }
    };

    match serde_json::to_string(&result) {
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
