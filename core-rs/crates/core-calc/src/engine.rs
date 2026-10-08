use core_common::{CoreError, CoreResult};
use crate::{basic, fraction, scientific, Rational};
use core_math::AngleMode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CalculatorMode { Basic, Scientific, Fraction }

#[derive(Debug, Clone, PartialEq)]
pub enum CalculationValue { Real(f64), Rational(Rational) }

#[derive(Debug, Clone, PartialEq)]
pub struct CalculationResult { pub value: CalculationValue }

pub struct CalculatorEngine;

impl CalculatorEngine {
    pub fn evaluate(mode: CalculatorMode, expression: &str) -> CoreResult<CalculationResult> {
        match mode {
            CalculatorMode::Basic => basic::evaluate(expression).map(|v| CalculationResult { value: CalculationValue::Real(v) }),
            CalculatorMode::Scientific => scientific::evaluate(expression).map(|v| CalculationResult { value: CalculationValue::Real(v) }),
            CalculatorMode::Fraction => fraction::evaluate(expression).map(|v| CalculationResult { value: CalculationValue::Rational(v) }),
        }
    }

    pub fn evaluate_with_angle(mode: CalculatorMode, expression: &str, angle: AngleMode) -> CoreResult<CalculationResult> {
        match mode {
            CalculatorMode::Basic => basic::evaluate(expression).map(|v| CalculationResult { value: CalculationValue::Real(v) }),
            CalculatorMode::Scientific => scientific::evaluate_with_angle(expression, angle).map(|v| CalculationResult { value: CalculationValue::Real(v) }),
            CalculatorMode::Fraction => fraction::evaluate(expression).map(|v| CalculationResult { value: CalculationValue::Rational(v) }),
        }
    }

    pub fn evaluate_real(mode: CalculatorMode, expression: &str) -> CoreResult<f64> {
        match Self::evaluate(mode, expression)?.value {
            CalculationValue::Real(v) => Ok(v),
            CalculationValue::Rational(v) => Ok(v.to_f64()),
        }
    }
}

pub(crate) fn finite(value: f64) -> CoreResult<f64> {
    value.is_finite().then_some(value).ok_or_else(|| CoreError::InvalidArgument("result is not finite".into()))
}
