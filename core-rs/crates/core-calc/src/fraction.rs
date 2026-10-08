use core_common::{CoreError, CoreResult};
use core_math::Rational;

pub fn evaluate(input: &str) -> CoreResult<Rational> {
    let mut parser = Parser { chars: input.trim().chars().peekable() };
    let value = parser.parse_add_sub()?;
    parser.skip_ws();
    if parser.chars.peek().is_some() {
        return Err(CoreError::Parse("unexpected token in fraction expression".into()));
    }
    Ok(value)
}

struct Parser<'a> {
    chars: std::iter::Peekable<std::str::Chars<'a>>,
}

impl<'a> Parser<'a> {
    fn skip_ws(&mut self) { while self.chars.peek().is_some_and(|c| c.is_whitespace()) { let _ = self.chars.next(); } }

    fn parse_add_sub(&mut self) -> CoreResult<Rational> {
        let mut value = self.parse_mul_div()?;
        loop {
            self.skip_ws();
            match self.chars.peek().copied() {
                Some('+') => { let _ = self.chars.next(); value = value.add(self.parse_mul_div()?)?; }
                Some('-') => { let _ = self.chars.next(); value = value.sub(self.parse_mul_div()?)?; }
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
                Some('*') | Some('×') => { let _ = self.chars.next(); value = value.mul(self.parse_unary()?)?; }
                Some('/') | Some('÷') => { let _ = self.chars.next(); value = value.div(self.parse_unary()?)?; }
                _ => break,
            }
        }
        Ok(value)
    }

    fn parse_unary(&mut self) -> CoreResult<Rational> {
        self.skip_ws();
        match self.chars.peek().copied() {
            Some('+') => { let _ = self.chars.next(); self.parse_unary() }
            Some('-') => {
                let _ = self.chars.next();
                let value = self.parse_unary()?;
                Rational::new(value.numerator.checked_neg().ok_or_else(|| CoreError::InvalidArgument("rational overflow".into()))?, value.denominator)
            }
            _ => self.parse_primary(),
        }
    }

    fn parse_primary(&mut self) -> CoreResult<Rational> {
        self.skip_ws();
        if self.chars.next_if_eq(&'(').is_some() {
            let value = self.parse_add_sub()?;
            self.skip_ws();
            if self.chars.next_if_eq(&')').is_none() {
                return Err(CoreError::Parse("missing ')'".into()));
            }
            return Ok(value);
        }

        let mut digits = String::new();
        while self.chars.peek().is_some_and(|c| c.is_ascii_digit()) {
            if let Some(ch) = self.chars.next() { digits.push(ch); }
        }
        if digits.is_empty() {
            return Err(CoreError::Parse("expected integer".into()));
        }
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
