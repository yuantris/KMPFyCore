mod lexer;
mod parser;

use core_common::{CoreError, CoreResult};

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Number(f64),
    Variable,
    VariableY,
    VariableT,
    Constant(f64),
    Unary { op: UnaryOp, expr: Box<Expr> },
    Binary { op: BinaryOp, left: Box<Expr>, right: Box<Expr> },
    Function { function: Function, expr: Box<Expr> },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp { Plus, Minus }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp { Add, Subtract, Multiply, Divide, Modulo, Power }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Function {
    Sin, Cos, Tan, Asin, Acos, Atan, Sqrt, Abs, Ln, Log, Exp, Floor, Ceil,
}

impl Expr {
    pub fn eval_x(&self, x: f64) -> f64 {
        match self {
            Self::Number(v) | Self::Constant(v) => *v,
            Self::Variable => x,
            Self::VariableY | Self::VariableT => f64::NAN,
            Self::Unary { op, expr } => match op {
                UnaryOp::Plus => expr.eval_x(x),
                UnaryOp::Minus => -expr.eval_x(x),
            },
            Self::Binary { op, left, right } => {
                let a = left.eval_x(x);
                let b = right.eval_x(x);
                match op {
                    BinaryOp::Add => a + b,
                    BinaryOp::Subtract => a - b,
                    BinaryOp::Multiply => a * b,
                    BinaryOp::Divide => a / b,
                    BinaryOp::Modulo => a % b,
                    BinaryOp::Power => a.powf(b),
                }
            }
            Self::Function { function, expr } => {
                let value = expr.eval_x(x);
                match function {
                    Function::Sin => value.sin(),
                    Function::Cos => value.cos(),
                    Function::Tan => value.tan(),
                    Function::Asin => value.asin(),
                    Function::Acos => value.acos(),
                    Function::Atan => value.atan(),
                    Function::Sqrt => value.sqrt(),
                    Function::Abs => value.abs(),
                    Function::Ln => value.ln(),
                    Function::Log => value.log10(),
                    Function::Exp => value.exp(),
                    Function::Floor => value.floor(),
                    Function::Ceil => value.ceil(),
                }
            }
        }
    }

    pub fn contains_variable(&self) -> bool {
        match self {
            Self::Variable | Self::VariableY | Self::VariableT => true,
            Self::Number(_) | Self::Constant(_) => false,
            Self::Unary { expr, .. } | Self::Function { expr, .. } => expr.contains_variable(),
            Self::Binary { left, right, .. } => {
                left.contains_variable() || right.contains_variable()
            }
        }
    }

    pub fn eval_constant(&self) -> CoreResult<f64> {
        if self.contains_variable() {
            return Err(CoreError::InvalidArgument(
                "expression contains variable x".into(),
            ));
        }
        let value = self.eval_x(0.0);
        if value.is_finite() {
            Ok(value)
        } else {
            Err(CoreError::InvalidArgument(
                "expression result is not finite".into(),
            ))
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Variable {
    X,
    Y,
    T,
}

impl Expr {
    pub fn eval_xy(&self, x: f64, y: f64) -> f64 { self.eval_vars(x, y, x) }

    pub fn eval_vars(&self, x: f64, y: f64, t: f64) -> f64 {
        match self {
            Self::Number(v) | Self::Constant(v) => *v,
            Self::Variable => x,
            Self::Unary { op, expr } => match op {
                UnaryOp::Plus => expr.eval_vars(x, y, t),
                UnaryOp::Minus => -expr.eval_vars(x, y, t),
            },
            Self::Binary { op, left, right } => {
                let a = left.eval_vars(x, y, t);
                let b = right.eval_vars(x, y, t);
                match op {
                    BinaryOp::Add => a + b, BinaryOp::Subtract => a - b,
                    BinaryOp::Multiply => a * b, BinaryOp::Divide => a / b,
                    BinaryOp::Modulo => a % b, BinaryOp::Power => a.powf(b),
                }
            }
            Self::Function { function, expr } => {
                let value = expr.eval_vars(x, y, t);
                match function {
                    Function::Sin => value.sin(), Function::Cos => value.cos(),
                    Function::Tan => value.tan(), Function::Asin => value.asin(),
                    Function::Acos => value.acos(), Function::Atan => value.atan(),
                    Function::Sqrt => value.sqrt(), Function::Abs => value.abs(),
                    Function::Ln => value.ln(), Function::Log => value.log10(),
                    Function::Exp => value.exp(), Function::Floor => value.floor(),
                    Function::Ceil => value.ceil(),
                }
            }
        }
    }
}

pub struct Expression;

impl Expression {
    pub fn compile(input: &str) -> CoreResult<Expr> {
        let input = input.trim();
        if input.is_empty() {
            return Err(CoreError::Parse("expression is empty".into()));
        }
        parser::parse(input)
    }

    pub fn eval(input: &str) -> CoreResult<f64> {
        Self::compile(input)?.eval_constant()
    }

    pub fn eval_x(input: &str, x: f64) -> CoreResult<f64> {
        Self::compile(input).map(|expr| expr.eval_x(x))
    }

    pub fn eval_vars(input: &str, x: f64, y: f64, t: f64) -> CoreResult<f64> {
        Self::compile(input).map(|expr| expr.eval_vars(x, y, t))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) {
        assert!((a - b).abs() < 1e-10, "{a} != {b}");
    }

    #[test]
    fn arithmetic() {
        close(Expression::eval("1 + 2 * 3").unwrap(), 7.0);
        close(Expression::eval("(1 + 2) * 3").unwrap(), 9.0);
        close(Expression::eval("2^3^2").unwrap(), 512.0);
        close(Expression::eval("-2^2").unwrap(), -4.0);
    }

    #[test]
    fn implicit_multiplication() {
        close(Expression::eval_x("2x + 2(x + 1)", 3.0).unwrap(), 14.0);
        close(Expression::eval_x("(x + 1)(x - 1)", 3.0).unwrap(), 8.0);
        close(
            Expression::eval_x("2sin(x)", std::f64::consts::PI / 2.0).unwrap(),
            2.0,
        );
    }

    #[test]
    fn functions() {
        close(Expression::eval("sqrt(9) + abs(-2) + pi - pi").unwrap(), 5.0);
        close(Expression::eval("log(100) + ln(exp(1))").unwrap(), 3.0);
    }

    #[test]
    fn variable() {
        close(Expression::eval_x("x^2 + 1", 3.0).unwrap(), 10.0);
        assert!(Expression::eval("x + 1").is_err());
    }

    #[test]
    fn invalid_domain() {
        assert!(!Expression::compile("sqrt(x)").unwrap().eval_x(-1.0).is_finite());
        assert!(!Expression::compile("1/x").unwrap().eval_x(0.0).is_finite());
    }
}
