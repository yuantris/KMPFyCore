use core_graph::{analyze, sample_checked, sample_parametric, sample_polar, GraphConfig};
use core_calc::{CalculatorEngine, CalculatorMode, CalculationValue};
use core_math::Expression;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_double, c_int};

fn input<'a>(ptr: *const c_char) -> Result<&'a str, String> {
    if ptr.is_null() { return Err("null string".into()); }
    unsafe { CStr::from_ptr(ptr).to_str().map_err(|e| e.to_string()) }
}

fn config(min_x: c_double, max_x: c_double, min_y: c_double, max_y: c_double, samples: c_int, width: c_int, height: c_int) -> Result<GraphConfig, String> {
    Ok(GraphConfig {
        min_x, max_x, min_y, max_y,
        samples: usize::try_from(samples).map_err(|_| "samples must be >= 0")?,
        pixel_width: u32::try_from(width).map_err(|_| "width must be >= 0")?,
        pixel_height: u32::try_from(height).map_err(|_| "height must be >= 0")?,
        ..GraphConfig::default()
    })
}

fn json_ptr<T: serde::Serialize>(value: T) -> *mut c_char {
    match serde_json::to_string(&value).ok().and_then(|s| CString::new(s).ok()) {
        Some(s) => s.into_raw(),
        None => std::ptr::null_mut(),
    }
}

fn graph_json(result: core_graph::GraphResult) -> Vec<Vec<[f64; 2]>> {
    result.segments.into_iter()
        .map(|s| s.points.into_iter().map(|p| [p.x, p.y]).collect())
        .collect()
}

#[no_mangle]
pub extern "C" fn core_rs_string_free(value: *mut c_char) {
    if !value.is_null() { unsafe { drop(CString::from_raw(value)); } }
}

#[no_mangle]
pub extern "C" fn core_rs_eval(expression: *const c_char) -> c_double {
    input(expression).ok().and_then(|s| Expression::eval(s).ok()).unwrap_or(f64::NAN)
}

#[no_mangle]
pub extern "C" fn core_rs_eval_x(expression: *const c_char, x: c_double) -> c_double {
    input(expression).ok().and_then(|s| Expression::eval_x(s, x).ok()).unwrap_or(f64::NAN)
}

#[no_mangle]
pub extern "C" fn core_rs_contains_variable(expression: *const c_char) -> bool {
    input(expression).ok().and_then(|s| Expression::compile(s).ok()).is_some_and(|e| e.contains_variable())
}

#[no_mangle]
pub extern "C" fn core_rs_calculate(expression: *const c_char, mode: c_int) -> *mut c_char {
    let result = (|| {
        let expression = input(expression)?;
        let mode = match mode {
            0 => CalculatorMode::Basic,
            1 => CalculatorMode::Scientific,
            2 => CalculatorMode::Fraction,
            _ => return Err("invalid calculator mode".to_string()),
        };
        let result = CalculatorEngine::evaluate(mode, expression).map_err(|e| e.to_string())?;
        let value = match result.value {
            CalculationValue::Real(v) => serde_json::json!({"type":"real","value":v}),
            CalculationValue::Rational(v) => serde_json::json!({"type":"rational","numerator":v.numerator,"denominator":v.denominator,"value":v.to_f64()}),
        };
        Ok::<_, String>(value)
    })();
    result.map_or(std::ptr::null_mut(), json_ptr)
}

#[no_mangle]
pub extern "C" fn core_rs_graph(
    expression: *const c_char, min_x: c_double, max_x: c_double, min_y: c_double, max_y: c_double,
    samples: c_int, width: c_int, height: c_int,
) -> *mut c_char {
    let result = (|| {
        let expr = Expression::compile(input(expression)?).map_err(|e| e.to_string())?;
        let cfg = config(min_x, max_x, min_y, max_y, samples, width, height).map_err(|e| e.to_string())?;
        Ok::<_, String>(graph_json(sample_checked(&expr, &cfg).map_err(|e| e.to_string())?))
    })();
    result.map_or(std::ptr::null_mut(), json_ptr)
}

#[no_mangle]
pub extern "C" fn core_rs_graph_polar(
    expression: *const c_char, min_x: c_double, max_x: c_double, min_y: c_double, max_y: c_double,
    min_t: c_double, max_t: c_double, samples: c_int, width: c_int, height: c_int,
) -> *mut c_char {
    let result = (|| {
        let expr = Expression::compile(input(expression)?).map_err(|e| e.to_string())?;
        let cfg = config(min_x, max_x, min_y, max_y, samples, width, height)?;
        Ok::<_, String>(graph_json(sample_polar(&expr, &cfg, min_t, max_t).map_err(|e| e.to_string())?))
    })();
    result.map_or(std::ptr::null_mut(), json_ptr)
}

#[no_mangle]
pub extern "C" fn core_rs_graph_parametric(
    x_expression: *const c_char, y_expression: *const c_char,
    min_x: c_double, max_x: c_double, min_y: c_double, max_y: c_double,
    min_t: c_double, max_t: c_double, samples: c_int, width: c_int, height: c_int,
) -> *mut c_char {
    let result = (|| {
        let x = Expression::compile(input(x_expression)?).map_err(|e| e.to_string())?;
        let y = Expression::compile(input(y_expression)?).map_err(|e| e.to_string())?;
        let cfg = config(min_x, max_x, min_y, max_y, samples, width, height)?;
        Ok::<_, String>(graph_json(sample_parametric(&x, &y, &cfg, min_t, max_t).map_err(|e| e.to_string())?))
    })();
    result.map_or(std::ptr::null_mut(), json_ptr)
}

#[no_mangle]
pub extern "C" fn core_rs_analyze(
    expression: *const c_char, min_x: c_double, max_x: c_double, min_y: c_double, max_y: c_double,
    samples: c_int, width: c_int, height: c_int,
) -> *mut c_char {
    let result = (|| {
        let expr = Expression::compile(input(expression)?).map_err(|e| e.to_string())?;
        let cfg = config(min_x, max_x, min_y, max_y, samples, width, height)?;
        Ok::<_, String>(analyze(&expr, &cfg).map_err(|e| e.to_string())?)
    })();
    result.map_or(std::ptr::null_mut(), json_ptr)
}

use core_binary::{ApkReader, ZipReader};
use core_search::SearchEngine;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

static SEARCH: OnceLock<Mutex<HashMap<u64, SearchEngine>>> = OnceLock::new();
static ZIPS: OnceLock<Mutex<HashMap<u64, ZipReader>>> = OnceLock::new();
fn search_map() -> &'static Mutex<HashMap<u64, SearchEngine>> { SEARCH.get_or_init(|| Mutex::new(HashMap::new())) }
fn zip_map() -> &'static Mutex<HashMap<u64, ZipReader>> { ZIPS.get_or_init(|| Mutex::new(HashMap::new())) }
fn next_handle(map_len: usize) -> u64 { map_len as u64 + 1 }

#[no_mangle]
pub extern "C" fn core_rs_search_create() -> u64 {
    let mut map = search_map().lock().unwrap_or_else(|e| e.into_inner());
    let mut id = next_handle(map.len());
    while map.contains_key(&id) { id += 1; }
    map.insert(id, SearchEngine::new());
    id
}
#[no_mangle]
pub extern "C" fn core_rs_search_destroy(handle: u64) {
    search_map().lock().unwrap_or_else(|e| e.into_inner()).remove(&handle);
}
#[no_mangle]
pub extern "C" fn core_rs_search_add(handle: u64, id: u64, text: *const c_char) -> bool {
    let Ok(text) = input(text) else { return false; };
    let mut map = search_map().lock().unwrap_or_else(|e| e.into_inner());
    map.get_mut(&handle).is_some_and(|engine| engine.add(id, text).is_ok())
}
#[no_mangle]
pub extern "C" fn core_rs_search_remove(handle: u64, id: u64) -> bool {
    search_map().lock().unwrap_or_else(|e| e.into_inner()).get_mut(&handle).is_some_and(|e| e.remove(id))
}
#[no_mangle]
pub extern "C" fn core_rs_search_clear(handle: u64) {
    if let Some(e) = search_map().lock().unwrap_or_else(|e| e.into_inner()).get_mut(&handle) { e.clear(); }
}
#[no_mangle]
pub extern "C" fn core_rs_search_size(handle: u64) -> usize {
    search_map().lock().unwrap_or_else(|e| e.into_inner()).get(&handle).map_or(0, SearchEngine::len)
}
#[no_mangle]
pub extern "C" fn core_rs_search(handle: u64, query: *const c_char, limit: usize) -> *mut c_char {
    let Ok(query) = input(query) else { return std::ptr::null_mut(); };
    let map = search_map().lock().unwrap_or_else(|e| e.into_inner());
    map.get(&handle).map_or(std::ptr::null_mut(), |e| json_ptr(e.search(query, limit)))
}

#[no_mangle]
pub extern "C" fn core_rs_zip_open(path: *const c_char) -> u64 {
    let Ok(path) = input(path) else { return 0; };
    let Ok(reader) = ZipReader::open(path) else { return 0; };
    let mut map = zip_map().lock().unwrap_or_else(|e| e.into_inner());
    let mut id = next_handle(map.len());
    while map.contains_key(&id) { id += 1; }
    map.insert(id, reader);
    id
}
#[no_mangle]
pub extern "C" fn core_rs_zip_close(handle: u64) {
    zip_map().lock().unwrap_or_else(|e| e.into_inner()).remove(&handle);
}
#[no_mangle]
pub extern "C" fn core_rs_zip_entries(handle: u64) -> *mut c_char {
    let map = zip_map().lock().unwrap_or_else(|e| e.into_inner());
    map.get(&handle).and_then(|e| e.entries().ok()).map_or(std::ptr::null_mut(), json_ptr)
}
#[no_mangle]
pub extern "C" fn core_rs_zip_contains(handle: u64, name: *const c_char) -> bool {
    let Ok(name) = input(name) else { return false; };
    let map = zip_map().lock().unwrap_or_else(|e| e.into_inner());
    map.get(&handle).and_then(|e| e.contains(name).ok()).unwrap_or(false)
}
#[no_mangle]
pub extern "C" fn core_rs_apk_is_apk(path: *const c_char) -> bool {
    let Ok(path) = input(path) else { return false; };
    ApkReader::open(path).and_then(|a| a.is_apk()).unwrap_or(false)
}
#[no_mangle]
pub extern "C" fn core_rs_apk_entries(path: *const c_char) -> *mut c_char {
    let Ok(path) = input(path) else { return std::ptr::null_mut(); };
    ApkReader::open(path).and_then(|a| a.entries()).map_or(std::ptr::null_mut(), json_ptr)
}
