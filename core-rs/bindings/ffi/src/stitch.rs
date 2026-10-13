use core_stitch::{stitch_files, StitchConfig};
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::path::PathBuf;

unsafe fn read_string<'a>(ptr: *const c_char) -> Result<&'a str, String> {
    if ptr.is_null() {
        return Err("null string argument".into());
    }
    unsafe { CStr::from_ptr(ptr).to_str().map_err(|error| error.to_string()) }
}

fn output_json(value: serde_json::Value) -> *mut c_char {
    CString::new(value.to_string())
        .map(CString::into_raw)
        .unwrap_or(std::ptr::null_mut())
}

/// Stitch files using JSON-encoded input paths and config. The returned JSON
/// is owned by Rust; release it with core_rs_string_free.
#[unsafe(no_mangle)]
pub extern "C" fn core_rs_stitch_files(
    input_paths_json: *const c_char,
    output_path: *const c_char,
    config_json: *const c_char,
) -> *mut c_char {
    let result = (|| {
        let input_text = unsafe { read_string(input_paths_json)? };
        let output_text = unsafe { read_string(output_path)? };
        let config_text = unsafe { read_string(config_json)? };
        let input_paths: Vec<String> = serde_json::from_str(input_text)
            .map_err(|error| format!("invalid input paths JSON: {error}"))?;
        if input_paths.is_empty() {
            return Err("at least one input image is required".to_string());
        }
        let config = if config_text.trim().is_empty() {
            StitchConfig::default()
        } else {
            serde_json::from_str::<StitchConfig>(config_text)
                .map_err(|error| format!("invalid stitch config JSON: {error}"))?
        };
        let paths: Vec<PathBuf> = input_paths.into_iter().map(PathBuf::from).collect();
        let metadata = stitch_files(&paths, std::path::Path::new(output_text), &config)
            .map_err(|error| error.to_string())?;
        serde_json::to_value(metadata).map_err(|error| error.to_string())
    })();
    match result {
        Ok(metadata) => output_json(serde_json::json!({"ok": true, "metadata": metadata})),
        Err(error) => output_json(serde_json::json!({"ok": false, "error": error})),
    }
}
