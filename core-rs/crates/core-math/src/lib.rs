use core_common::{CoreError, CoreResult};

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Number(f64), Identifier(String), Plus, Minus, Star, Slash, Percent, Caret,
    LeftParen, RightParen, End,
}

pub struct Expression;

impl Expression {
    pub fn eval(input: &str) -> CoreResult<f64> {
        let mut parser = Parser::new(tokenize(input)?);
        let value = parser.parse_expression()?;
        if parser.peek() != &Token::End {
            return Err(CoreError::Parse("unexpected token".into()));
        }
        Ok(value)
    }
}

fn tokenize(input: &str) -> CoreResult<Vec<Token>> {
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;
    let mut tokens = Vec::new();
    while i < chars.len() {
        let c = chars[i];
        if c.is_whitespace() { i += 1; continue; }
        if c.is_ascii_digit() || c == '.' {
            let start = i;
            i += 1;
            while i < chars.len() && (chars[i].is_ascii_digit() || matches!(chars[i], '.' | 'e' | 'E')) {
                i += 1;
            }
            if i < chars.len() && matches!(chars[i], '+' | '-') && i > start && matches!(chars[i - 1], 'e' | 'E') {
                i += 1;
                while i < chars.len() && chars[i].is_ascii_digit() { i += 1; }
            }
            let text: String = chars[start..i].iter().collect();
            let number = text.parse::<f64>().map_err(|_| CoreError::Parse(format!("invalid number: {text}")))?;
            tokens.push(Token::Number(number));
            continue;
        }
        if c.is_ascii_alphabetic() || c == '_' {
            let start = i;
            i += 1;
            while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') { i += 1; }
            tokens.push(Token::Identifier(chars[start..i].iter().collect::<String>().to_lowercase()));
            continue;
        }
        let token = match c {
            '+' => Token::Plus, '-' => Token::Minus, '*' => Token::Star, '/' => Token::Slash,
            '%' => Token::Percent, '^' => Token::Caret, '(' => Token::LeftParen, ')' => Token::RightParen,
            _ => return Err(CoreError::Parse(format!("unexpected character: {c}"))),
        };
        tokens.push(token);
        i += 1;
    }
    tokens.push(Token::End);
    Ok(tokens)
}

struct Parser { tokens: Vec<Token>, index: usize }

impl Parser {
    fn new(tokens: Vec<Token>) -> Self { Self { tokens, index: 0 } }
    fn peek(&self) -> &Token { &self.tokens[self.index] }
    fn consume(&mut self) -> Token { let t = self.tokens[self.index].clone(); self.index += 1; t }

    fn parse_expression(&mut self) -> CoreResult<f64> {
        let mut value = self.parse_term()?;
        loop {
            match self.peek() {
                Token::Plus => { self.consume(); value += self.parse_term()?; }
                Token::Minus => { self.consume(); value -= self.parse_term()?; }
                _ => break,
            }
        }
        Ok(value)
    }

    fn parse_term(&mut self) -> CoreResult<f64> {
        let mut value = self.parse_power()?;
        loop {
            match self.peek() {
                Token::Star => { self.consume(); value *= self.parse_power()?; }
                Token::Slash => {
                    self.consume(); let rhs = self.parse_power()?;
                    if rhs == 0.0 { return Err(CoreError::InvalidArgument("division by zero".into())); }
                    value /= rhs;
                }
                Token::Percent => {
                    self.consume(); let rhs = self.parse_power()?;
                    if rhs == 0.0 { return Err(CoreError::InvalidArgument("modulo by zero".into())); }
                    value %= rhs;
                }
                _ => break,
            }
        }
        Ok(value)
    }

    fn parse_power(&mut self) -> CoreResult<f64> {
        let base = self.parse_unary()?;
        if self.peek() == &Token::Caret {
            self.consume();
            return Ok(base.powf(self.parse_power()?));
        }
        Ok(base)
    }

    fn parse_unary(&mut self) -> CoreResult<f64> {
        match self.peek() {
            Token::Plus => { self.consume(); self.parse_unary() }
            Token::Minus => { self.consume(); Ok(-self.parse_unary()?) }
            _ => self.parse_primary(),
        }
    }

    fn parse_primary(&mut self) -> CoreResult<f64> {
        match self.consume() {
            Token::Number(v) => Ok(v),
            Token::Identifier(name) => {
                if self.peek() == &Token::LeftParen {
                    self.consume();
                    let value = self.parse_expression()?;
                    if self.peek() != &Token::RightParen { return Err(CoreError::Parse("expected ')'".into())); }
                    self.consume();
                    apply_function(&name, value)
                } else {
                    match name.as_str() {
                        "pi" => Ok(std::f64::consts::PI),
                        "e" => Ok(std::f64::consts::E),
                        _ => Err(CoreError::Parse(format!("unknown identifier: {name}"))),
                    }
                }
            }
            Token::LeftParen => {
                let value = self.parse_expression()?;
                if self.peek() != &Token::RightParen { return Err(CoreError::Parse("expected ')'".into())); }
                self.consume();
                Ok(value)
            }
            _ => Err(CoreError::Parse("expected value".into())),
        }
    }
}

fn apply_function(name: &str, value: f64) -> CoreResult<f64> {
    Ok(match name {
        "sin" => value.sin(), "cos" => value.cos(), "tan" => value.tan(),
        "sqrt" => value.sqrt(), "abs" => value.abs(), "ln" => value.ln(), "log" => value.log10(),
        _ => return Err(CoreError::Parse(format!("unknown function: {name}"))),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn arithmetic() { assert_eq!(Expression::eval("1 + 2 * 3").unwrap(), 7.0); }
    #[test] fn parentheses() { assert_eq!(Expression::eval("(1 + 2) * 3").unwrap(), 9.0); }
    #[test] fn function() { assert_eq!(Expression::eval("sqrt(9) + abs(-2)").unwrap(), 5.0); }
    #[test] fn constants() { assert!((Expression::eval("pi * 2").unwrap() - std::f64::consts::PI * 2.0).abs() < 0.00001); }
}
