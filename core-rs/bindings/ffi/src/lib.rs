use core_graph::{analyze, sample_checked, sample_parametric, sample_polar, GraphConfig};
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
