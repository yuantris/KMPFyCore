use core_math::Expression;
use jni::objects::{JClass, JString};
use jni::sys::{jboolean, jdouble};
use jni::JNIEnv;
use crate::support::{string_arg, throw};

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
pub extern "system" fn Java_io_core_rs_CoreRsNative_nativeExpressionEvalX(
    mut env: JNIEnv, _class: JClass, expression: JString, x: jdouble,
) -> jdouble {
    let Ok(expression) = string_arg(&mut env, expression) else { return f64::NAN; };
    match Expression::eval_x(&expression, x) {
        Ok(value) => value,
        Err(error) => {
            throw(&mut env, error.to_string());
            f64::NAN
        }
    }
}


#[unsafe(no_mangle)]
pub extern "system" fn Java_io_core_rs_CoreRsNative_nativeExpressionContainsVariable(
    mut env: JNIEnv, _class: JClass, expression: JString,
) -> jboolean {
    let Ok(expression) = string_arg(&mut env, expression) else { return 0; };
    match Expression::compile(&expression) {
        Ok(expr) => expr.contains_variable() as jboolean,
        Err(error) => {
            throw(&mut env, error.to_string());
            0
        }
    }
}
