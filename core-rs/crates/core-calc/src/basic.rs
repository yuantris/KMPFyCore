use core_common::{CoreError, CoreResult};
use core_math::{BinaryOp, Expr, Expression, Function, UnaryOp};

pub fn evaluate(expression: &str) -> CoreResult<f64> {
    let expr = Expression::compile(expression)?;
    validate(&expr)?;
    let value = expr.eval_constant()?;
    value.is_finite().then_some(value).ok_or_else(|| CoreError::InvalidArgument("result is not finite".into()))
}

fn validate(expr: &Expr) -> CoreResult<()> {
    match expr {
        Expr::Number(_) | Expr::Constant(_) => Ok(()),
        Expr::Unary { op: UnaryOp::Plus | UnaryOp::Minus, expr } => validate(expr),
        Expr::Binary { op, left, right } => {
            match op {
                BinaryOp::Add | BinaryOp::Subtract | BinaryOp::Multiply | BinaryOp::Divide | BinaryOp::Modulo => {}
                BinaryOp::Power => return Err(CoreError::InvalidArgument("basic calculator does not support power".into())),
            }
            validate(left)?;
            validate(right)
        }
        Expr::Variable | Expr::VariableY | Expr::VariableT => Err(CoreError::InvalidArgument("basic calculator does not support variables".into())),
        Expr::Function { function: Function::Percent, expr } => validate(expr),
        Expr::Function { .. } => Err(CoreError::InvalidArgument("basic calculator does not support scientific functions".into())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn arithmetic_only() {
        assert_eq!(evaluate("1+2*3").unwrap(), 7.0);
        assert!(evaluate("sin(1)").is_err());
        assert!(evaluate("2^3").is_err());
    }
}
