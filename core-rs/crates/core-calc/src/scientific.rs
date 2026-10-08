use core_common::CoreResult;
use core_math::Expression;

pub fn evaluate(expression: &str) -> CoreResult<f64> {
    let value = Expression::eval(expression)?;
    value.is_finite()
        .then_some(value)
        .ok_or_else(|| core_common::CoreError::InvalidArgument("result is not finite".into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn functions() {
        assert!((evaluate("sin(pi/2)+sqrt(9)").unwrap() - 4.0).abs() < 1e-10);
    }
}
