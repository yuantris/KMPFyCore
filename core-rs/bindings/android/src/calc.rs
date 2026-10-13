use core_calc::{CalculationValue, CalculatorEngine, CalculatorMode};
use core_math::AngleMode;
use jni::objects::{JClass, JString};
use jni::sys::jstring;
use jni::JNIEnv;
use crate::support::{string_arg, throw};

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_core_rs_CoreRsNative_nativeCalculate(
    mut env: JNIEnv, _class: JClass, expression: JString, mode: i32, angle: i32,
) -> jstring {
    let expression = match string_arg(&mut env, expression) {
        Ok(value) => value,
        Err(_) => return std::ptr::null_mut(),
    };
    let mode = match mode {
        0 => CalculatorMode::Basic,
        1 => CalculatorMode::Scientific,
        2 => CalculatorMode::Fraction,
        _ => {
            throw(&mut env, "invalid calculator mode".to_string());
            return std::ptr::null_mut();
        }
    };
    let angle = match angle { 0 => AngleMode::Rad, 1 => AngleMode::Deg, _ => { throw(&mut env, "invalid angle mode".to_string()); return std::ptr::null_mut(); } };
    match CalculatorEngine::evaluate_with_angle(mode, &expression, angle) {
        Ok(result) => {
            let json = match result.value {
                CalculationValue::Real(value) => format!(r#"{{"type":"real","value":{value}}}"#),
                CalculationValue::Rational(value) => format!(r#"{{"type":"rational","numerator":{},"denominator":{},"value":{}}}"#, value.numerator, value.denominator, value.to_f64()),
            };
            match env.new_string(json) {
                Ok(value) => value.into_raw(),
                Err(error) => { throw(&mut env, error.to_string()); std::ptr::null_mut() }
            }
        }
        Err(error) => {
            throw(&mut env, error.to_string());
            std::ptr::null_mut()
        }
    }
}
