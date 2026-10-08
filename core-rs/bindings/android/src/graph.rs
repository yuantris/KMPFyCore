use core_graph::{GraphConfig, GraphResult};
use core_math::Expression;
use jni::objects::{JClass, JString};
use jni::sys::{jdouble, jint, jstring};
use jni::JNIEnv;
use crate::support::throw;

fn graph_json(result: GraphResult) -> Result<String, serde_json::Error> {
    let payload: Vec<Vec<[f64; 2]>> = result.segments.into_iter()
        .map(|segment| segment.points.into_iter().map(|point| [point.x, point.y]).collect())
        .collect();
    serde_json::to_string(&payload)
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
pub extern "system" fn Java_io_core_rs_CoreRsNative_nativeGraphAdvanced(
    mut env: JNIEnv, _class: JClass, expression: JString, second_expression: JString,
    mode: jint, min_x: jdouble, max_x: jdouble, min_y: jdouble, max_y: jdouble,
    min_t: jdouble, max_t: jdouble, samples: jint, pixel_width: jint, pixel_height: jint,
) -> jstring {
    let Ok(expression) = string_arg(&mut env, expression) else { return std::ptr::null_mut(); };
    let Ok(second) = string_arg(&mut env, second_expression) else { return std::ptr::null_mut(); };
    let first = match Expression::compile(&expression) {
        Ok(v) => v, Err(e) => { throw(&mut env, e.to_string()); return std::ptr::null_mut(); }
    };
    let config = GraphConfig {
        min_x, max_x, min_y, max_y,
        samples: samples.max(64) as usize,
        pixel_width: pixel_width.max(2) as u32,
        pixel_height: pixel_height.max(2) as u32,
        ..GraphConfig::default()
    };
    let result = match mode {
        0 => core_graph::sample_checked(&first, &config),
        1 => core_graph::sample_polar(&first, &config, min_t, max_t),
        2 => {
            let second = match Expression::compile(&second) {
                Ok(v) => v, Err(e) => { throw(&mut env, e.to_string()); return std::ptr::null_mut(); }
            };
            core_graph::sample_parametric(&first, &second, &config, min_t, max_t)
        }
        _ => {
            throw(&mut env, "invalid graph mode");
            return std::ptr::null_mut();
        }
    };
    let result = match result {
        Ok(v) => v, Err(e) => { throw(&mut env, e.to_string()); return std::ptr::null_mut(); }
    };
    match graph_json(result) {
        Ok(json) => match env.new_string(json) {
            Ok(value) => value.into_raw(),
            Err(e) => { throw(&mut env, e.to_string()); std::ptr::null_mut() }
        },
        Err(e) => { throw(&mut env, e.to_string()); std::ptr::null_mut() }
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_core_rs_CoreRsNative_nativeGraphAnalyze(
    mut env: JNIEnv, _class: JClass, expression: JString,
    min_x: jdouble, max_x: jdouble, min_y: jdouble, max_y: jdouble, samples: jint,
) -> jstring {
    let Ok(expression) = string_arg(&mut env, expression) else { return std::ptr::null_mut(); };
    let first = match Expression::compile(&expression) {
        Ok(v) => v, Err(e) => { throw(&mut env, e.to_string()); return std::ptr::null_mut(); }
    };
    let config = GraphConfig {
        min_x, max_x, min_y, max_y,
        samples: samples.max(64) as usize,
        ..GraphConfig::default()
    };
    let analysis = match core_graph::analyze(&first, &config) {
        Ok(v) => v, Err(e) => { throw(&mut env, e.to_string()); return std::ptr::null_mut(); }
    };
    match serde_json::to_string(&analysis) {
        Ok(json) => match env.new_string(json) {
            Ok(value) => value.into_raw(),
            Err(e) => { throw(&mut env, e.to_string()); std::ptr::null_mut() }
        },
        Err(e) => { throw(&mut env, e.to_string()); std::ptr::null_mut() }
    }
}
