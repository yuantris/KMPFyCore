use core_common::{CoreError, CoreResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Rational { pub numerator: i64, pub denominator: i64 }

impl Rational {
    pub fn new(numerator: i64, denominator: i64) -> CoreResult<Self> {
        if denominator == 0 { return Err(CoreError::InvalidArgument("fraction denominator must not be zero".into())); }
        let sign = if denominator < 0 { -1 } else { 1 };
        let n = numerator.checked_mul(sign).ok_or_else(|| CoreError::InvalidArgument("fraction overflow".into()))?;
        let d = denominator.checked_mul(sign).ok_or_else(|| CoreError::InvalidArgument("fraction overflow".into()))?;
        let g = gcd(n.unsigned_abs(), d.unsigned_abs());
        Ok(Self { numerator: n / g as i64, denominator: d / g as i64 })
    }

    pub fn integer(value: i64) -> Self { Self { numerator: value, denominator: 1 } }
    pub fn to_f64(self) -> f64 { self.numerator as f64 / self.denominator as f64 }

    fn add(self, other: Self) -> CoreResult<Self> {
        Self::new(
            self.numerator.checked_mul(other.denominator).and_then(|a| other.numerator.checked_mul(self.denominator).and_then(|b| a.checked_add(b))).ok_or_else(|| CoreError::InvalidArgument("fraction overflow".into()))?,
            self.denominator.checked_mul(other.denominator).ok_or_else(|| CoreError::InvalidArgument("fraction overflow".into()))?,
        )
    }
    fn sub(self, other: Self) -> CoreResult<Self> { self.add(Self::new(other.numerator.checked_neg().ok_or_else(|| CoreError::InvalidArgument("fraction overflow".into()))?, other.denominator)?) }
    fn mul(self, other: Self) -> CoreResult<Self> {
        Self::new(
            self.numerator.checked_mul(other.numerator).ok_or_else(|| CoreError::InvalidArgument("fraction overflow".into()))?,
            self.denominator.checked_mul(other.denominator).ok_or_else(|| CoreError::InvalidArgument("fraction overflow".into()))?,
        )
    }
    fn div(self, other: Self) -> CoreResult<Self> {
        if other.numerator == 0 { return Err(CoreError::InvalidArgument("division by zero".into())); }
        Self::new(
            self.numerator.checked_mul(other.denominator).ok_or_else(|| CoreError::InvalidArgument("fraction overflow".into()))?,
            self.denominator.checked_mul(other.numerator).ok_or_else(|| CoreError::InvalidArgument("fraction overflow".into()))?,
        )
    }
}

fn gcd(mut a: u64, mut b: u64) -> u64 { while b != 0 { let r = a % b; a = b; b = r; } a.max(1) }

pub fn evaluate(input: &str) -> CoreResult<Rational> {
    let mut parser = Parser { chars: input.trim().chars().peekable() };
    let value = parser.parse_add_sub()?;
    parser.skip_ws();
    if parser.chars.peek().is_some() { return Err(CoreError::Parse("unexpected token in fraction expression".into())); }
    Ok(value)
}

struct Parser<'a> { chars: std::iter::Peekable<std::str::Chars<'a>> }
impl<'a> Parser<'a> {
    fn skip_ws(&mut self) { while self.chars.peek().is_some_and(|c| c.is_whitespace()) { let _ = self.chars.next(); } }
    fn parse_add_sub(&mut self) -> CoreResult<Rational> {
        let mut value = self.parse_mul_div()?;
        loop {
            self.skip_ws();
            match self.chars.peek().copied() {
                Some('+') => { self.chars.next(); value = value.add(self.parse_mul_div()?)?; }
                Some('-') => { self.chars.next(); value = value.sub(self.parse_mul_div()?)?; }
                _ => break,
            }
        }
        Ok(value)
    }
    fn parse_mul_div(&mut self) -> CoreResult<Rational> {
        let mut value = self.parse_unary()?;
        loop {
            self.skip_ws();
            match self.chars.peek().copied() {
                Some('*') | Some('×') => { self.chars.next(); value = value.mul(self.parse_unary()?)?; }
                Some('/') | Some('÷') => { self.chars.next(); value = value.div(self.parse_unary()?)?; }
                _ => break,
            }
        }
        Ok(value)
    }
    fn parse_unary(&mut self) -> CoreResult<Rational> {
        self.skip_ws();
        match self.chars.peek().copied() {
            Some('+') => { self.chars.next(); self.parse_unary() }
            Some('-') => { self.chars.next(); let v = self.parse_unary()?; Rational::new(-v.numerator, v.denominator) }
            _ => self.parse_primary(),
        }
    }
    fn parse_primary(&mut self) -> CoreResult<Rational> {
        self.skip_ws();
        if self.chars.next_if_eq(&'(').is_some() {
            let value = self.parse_add_sub()?;
            self.skip_ws();
            if self.chars.next_if_eq(&')').is_none() { return Err(CoreError::Parse("missing ')'".into())); }
            return Ok(value);
        }
        let mut digits = String::new();
        while self.chars.peek().is_some_and(|c| c.is_ascii_digit()) { digits.push(self.chars.next().unwrap_or(' ')); }
        if digits.is_empty() { return Err(CoreError::Parse("expected integer".into())); }
        let n = digits.parse::<i64>().map_err(|_| CoreError::Parse("integer is out of range".into()))?;
        Ok(Rational::integer(n))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exact_arithmetic() {
        assert_eq!(evaluate("1/2 + 1/6").unwrap(), Rational::new(2, 3).unwrap());
        assert_eq!(evaluate("(3/4)*2").unwrap(), Rational::new(3, 2).unwrap());
        assert_eq!(evaluate("2 - 5/3").unwrap(), Rational::new(1, 3).unwrap());
    }
}
