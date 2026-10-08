use core_common::{CoreError, CoreResult};
use super::{BinaryOp, Expr, Function, UnaryOp};
use super::lexer::{tokenize, Token};
pub(crate) fn parse(input: &str) -> CoreResult<Expr> {
    let mut parser = Parser::new(tokenize(input)?); let expr = parser.parse_expression()?;
    if parser.peek() != &Token::End { return Err(CoreError::Parse(format!("unexpected token: {:?}", parser.peek()))); }
    Ok(expr)
}
struct Parser { tokens: Vec<Token>, index: usize }
impl Parser {
    fn new(tokens: Vec<Token>) -> Self { Self { tokens, index: 0 } }
    fn peek(&self) -> &Token { &self.tokens[self.index] }
    fn consume(&mut self) -> Token { let t = self.tokens[self.index].clone(); self.index += 1; t }
    fn parse_expression(&mut self) -> CoreResult<Expr> { self.parse_additive() }
    fn parse_additive(&mut self) -> CoreResult<Expr> {
        let mut expr = self.parse_multiplicative()?;
        loop { let op = match self.peek() { Token::Plus => BinaryOp::Add, Token::Minus => BinaryOp::Subtract, _ => break }; self.consume(); let rhs = self.parse_multiplicative()?; expr = Expr::Binary { op, left: Box::new(expr), right: Box::new(rhs) }; }
        Ok(expr)
    }
    fn parse_multiplicative(&mut self) -> CoreResult<Expr> {
        let mut expr = self.parse_unary()?;
        loop {
            let op = match self.peek() { Token::Star => Some(BinaryOp::Multiply), Token::Slash => Some(BinaryOp::Divide), Token::Percent => Some(BinaryOp::Modulo), t if starts_primary(t) => Some(BinaryOp::Multiply), _ => None };
            let Some(op) = op else { break };
            if matches!(self.peek(), Token::Star | Token::Slash | Token::Percent) { self.consume(); }
            let rhs = self.parse_unary()?; expr = Expr::Binary { op, left: Box::new(expr), right: Box::new(rhs) };
        }
        Ok(expr)
    }
    // Unary is below power: -2^2 == -(2^2).
    fn parse_unary(&mut self) -> CoreResult<Expr> {
        match self.peek() {
            Token::Plus => { self.consume(); Ok(Expr::Unary { op: UnaryOp::Plus, expr: Box::new(self.parse_unary()?) }) }
            Token::Minus => { self.consume(); Ok(Expr::Unary { op: UnaryOp::Minus, expr: Box::new(self.parse_unary()?) }) }
            _ => self.parse_power(),
        }
    }
    // Exponentiation is right associative through parse_unary on the exponent.
    fn parse_power(&mut self) -> CoreResult<Expr> {
        let mut base = self.parse_primary()?;
        if self.peek() == &Token::Caret { self.consume(); let exponent = self.parse_unary()?; base = Expr::Binary { op: BinaryOp::Power, left: Box::new(base), right: Box::new(exponent) }; }
        loop {
            match self.peek() {
                Token::Percent => { self.consume(); base = Expr::Function { function: Function::Percent, expr: Box::new(base) }; }
                Token::Factorial => { self.consume(); base = Expr::Function { function: Function::Factorial, expr: Box::new(base) }; }
                _ => break,
            }
        }
        Ok(base)
    }
    fn parse_primary(&mut self) -> CoreResult<Expr> {
        match self.consume() {
            Token::Number(v) => Ok(Expr::Number(v)),
            Token::Identifier(name) => {
                if self.peek() == &Token::LeftParen {
                    self.consume(); let argument = self.parse_expression()?;
                    if self.peek() != &Token::RightParen { return Err(CoreError::Parse(format!("expected ')' after {name}"))); }
                    self.consume(); Ok(Expr::Function { function: parse_function(&name)?, expr: Box::new(argument) })
                } else { match name.as_str() { "x" => Ok(Expr::Variable), "y" => Ok(Expr::VariableY), "t" => Ok(Expr::VariableT), "pi" => Ok(Expr::Constant(std::f64::consts::PI)), "e" => Ok(Expr::Constant(std::f64::consts::E)), _ => Err(CoreError::Parse(format!("unknown identifier: {name}"))) } }
            }
            Token::LeftParen => { let expr = self.parse_expression()?; if self.peek() != &Token::RightParen { return Err(CoreError::Parse("expected ')'".into())); } self.consume(); Ok(expr) }
            token => Err(CoreError::Parse(format!("expected value, found {token:?}"))),
        }
    }
}
fn starts_primary(token: &Token) -> bool { matches!(token, Token::Number(_) | Token::Identifier(_) | Token::LeftParen) }
fn parse_function(name: &str) -> CoreResult<Function> {
    match name { "sin" => Ok(Function::Sin), "cos" => Ok(Function::Cos), "tan" => Ok(Function::Tan), "asin" => Ok(Function::Asin), "acos" => Ok(Function::Acos), "atan" => Ok(Function::Atan), "sqrt" => Ok(Function::Sqrt), "abs" => Ok(Function::Abs), "ln" => Ok(Function::Ln), "log" | "log10" => Ok(Function::Log), "exp" => Ok(Function::Exp), "floor" => Ok(Function::Floor), "ceil" => Ok(Function::Ceil), "fact" | "factorial" => Ok(Function::Factorial), "inv" | "reciprocal" => Ok(Function::Reciprocal), _ => Err(CoreError::Parse(format!("unknown function: {name}"))) }
}
