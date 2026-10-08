use core_graph::{GraphConfig, GraphResult};
use core_math::Expression;
use jni::objects::{JClass, JString};
use jni::sys::{jdouble, jint, jstring};
use jni::JNIEnv;
use crate::support::{string_arg, throw};

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
