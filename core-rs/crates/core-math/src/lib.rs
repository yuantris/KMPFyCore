use core_common::{CoreError, CoreResult};

mod graph;
pub use graph::{sample as sample_graph, GraphConfig, GraphPoint, GraphSegment};

#[derive(Debug, Clone, PartialEq)]
enum Token { Number(f64), Identifier(String), Plus, Minus, Star, Slash, Percent, Caret, LeftParen, RightParen, End }

#[derive(Debug, Clone)]
pub enum Expr { Number(f64), Variable, Constant(f64), Unary { op: UnaryOp, value: Box<Expr> }, Binary { op: BinaryOp, left: Box<Expr>, right: Box<Expr> }, Function { name: Function, argument: Box<Expr> } }
#[derive(Debug, Clone, Copy)] pub enum UnaryOp { Plus, Minus }
#[derive(Debug, Clone, Copy)] pub enum BinaryOp { Add, Subtract, Multiply, Divide, Modulo, Power }
#[derive(Debug, Clone, Copy)] pub enum Function { Sin, Cos, Tan, Asin, Acos, Atan, Sqrt, Abs, Ln, Log, Exp, Floor, Ceil }

impl Expr {
    pub fn eval_x(&self, x: f64) -> f64 {
        match self {
            Self::Number(v) | Self::Constant(v) => *v,
            Self::Variable => x,
            Self::Unary { op, value } => match op { UnaryOp::Plus => value.eval_x(x), UnaryOp::Minus => -value.eval_x(x) },
            Self::Binary { op, left, right } => { let a=left.eval_x(x); let b=right.eval_x(x); match op { BinaryOp::Add=>a+b, BinaryOp::Subtract=>a-b, BinaryOp::Multiply=>a*b, BinaryOp::Divide=>a/b, BinaryOp::Modulo=>a%b, BinaryOp::Power=>a.powf(b) } }
            Self::Function { name, argument } => { let v=argument.eval_x(x); match name { Function::Sin=>v.sin(), Function::Cos=>v.cos(), Function::Tan=>v.tan(), Function::Asin=>v.asin(), Function::Acos=>v.acos(), Function::Atan=>v.atan(), Function::Sqrt=>v.sqrt(), Function::Abs=>v.abs(), Function::Ln=>v.ln(), Function::Log=>v.log10(), Function::Exp=>v.exp(), Function::Floor=>v.floor(), Function::Ceil=>v.ceil() } }
        }
    }
}
pub struct Expression;
impl Expression {
    pub fn compile(input:&str)->CoreResult<Expr>{ let mut p=Parser::new(tokenize(input)?); let e=p.parse_expression()?; if p.peek()!=&Token::End{return Err(CoreError::Parse("unexpected token".into()))} Ok(e) }
    pub fn eval(input:&str)->CoreResult<f64>{Self::compile(input).map(|e|e.eval_x(0.0))}
    pub fn graph(input:&str,config:GraphConfig)->CoreResult<Vec<GraphSegment>>{Self::compile(input).map(|e|sample_graph(&e,&config))}
}
fn tokenize(input:&str)->CoreResult<Vec<Token>>{
    let c:Vec<char>=input.chars().collect(); let mut i=0; let mut t=Vec::new();
    while i<c.len(){let ch=c[i]; if ch.is_whitespace(){i+=1;continue}
        if ch.is_ascii_digit()||ch=='.'{let s=i;i+=1;while i<c.len()&&(c[i].is_ascii_digit()||c[i]=='.'){i+=1}
            if i<c.len()&&matches!(c[i],'e'|'E'){i+=1;if i<c.len()&&matches!(c[i],'+'|'-'){i+=1}let es=i;while i<c.len()&&c[i].is_ascii_digit(){i+=1}if es==i{return Err(CoreError::Parse("invalid exponent".into()))}}
            let text:String=c[s..i].iter().collect();t.push(Token::Number(text.parse().map_err(|_|CoreError::Parse(format!("invalid number: {text}")))?));continue}
        if ch.is_ascii_alphabetic()||ch=='_'{let s=i;i+=1;while i<c.len()&&(c[i].is_ascii_alphanumeric()||c[i]=='_'){i+=1}t.push(Token::Identifier(c[s..i].iter().collect::<String>().to_lowercase()));continue}
        t.push(match ch{'+' =>Token::Plus,'-'=>Token::Minus,'*'=>Token::Star,'/'=>Token::Slash,'%'=>Token::Percent,'^'=>Token::Caret,'('=>Token::LeftParen,')'=>Token::RightParen,_=>return Err(CoreError::Parse(format!("unexpected character: {ch}")))});i+=1}
    t.push(Token::End);Ok(t)
}
struct Parser{tokens:Vec<Token>,index:usize}
impl Parser{
 fn new(tokens:Vec<Token>)->Self{Self{tokens,index:0}} fn peek(&self)->&Token{&self.tokens[self.index]} fn consume(&mut self)->Token{let t=self.tokens[self.index].clone();self.index+=1;t}
 fn parse_expression(&mut self)->CoreResult<Expr>{let mut v=self.parse_term()?;loop{v=match self.peek(){Token::Plus=>{self.consume();Expr::Binary{op:BinaryOp::Add,left:Box::new(v),right:Box::new(self.parse_term()?)}}Token::Minus=>{self.consume();Expr::Binary{op:BinaryOp::Subtract,left:Box::new(v),right:Box::new(self.parse_term()?)}}_ =>break}}Ok(v)}
 fn parse_term(&mut self)->CoreResult<Expr>{let mut v=self.parse_power()?;loop{v=match self.peek(){Token::Star=>{self.consume();Expr::Binary{op:BinaryOp::Multiply,left:Box::new(v),right:Box::new(self.parse_power()?)}}Token::Slash=>{self.consume();Expr::Binary{op:BinaryOp::Divide,left:Box::new(v),right:Box::new(self.parse_power()?)}}Token::Percent=>{self.consume();Expr::Binary{op:BinaryOp::Modulo,left:Box::new(v),right:Box::new(self.parse_power()?)}}Token::Number(_)|Token::Identifier(_)|Token::LeftParen=>Expr::Binary{op:BinaryOp::Multiply,left:Box::new(v),right:Box::new(self.parse_power()?)},_= >break}}Ok(v)}
 fn parse_power(&mut self)->CoreResult<Expr>{let b=self.parse_unary()?;if self.peek()==&Token::Caret{self.consume();return Ok(Expr::Binary{op:BinaryOp::Power,left:Box::new(b),right:Box::new(self.parse_power()?)})}Ok(b)}
 fn parse_unary(&mut self)->CoreResult<Expr>{match self.peek(){Token::Plus=>{self.consume();Ok(Expr::Unary{op:UnaryOp::Plus,value:Box::new(self.parse_unary()?)})}Token::Minus=>{self.consume();Ok(Expr::Unary{op:UnaryOp::Minus,value:Box::new(self.parse_unary()?)})}_=>self.parse_primary()}}
 fn parse_primary(&mut self)->CoreResult<Expr>{match self.consume(){Token::Number(v)=>Ok(Expr::Number(v)),Token::Identifier(n)=>{if self.peek()==&Token::LeftParen{self.consume();let v=self.parse_expression()?;if self.peek()!=&Token::RightParen{return Err(CoreError::Parse("expected ')'".into()))}self.consume();Ok(Expr::Function{name:parse_function(&n)?,argument:Box::new(v)})}else{match n.as_str(){"x"=>Ok(Expr::Variable),"pi"=>Ok(Expr::Constant(std::f64::consts::PI)),"e"=>Ok(Expr::Constant(std::f64::consts::E)),_= >Err(CoreError::Parse(format!("unknown identifier: {n}")))}}}Token::LeftParen=>{let v=self.parse_expression()?;if self.peek()!=&Token::RightParen{return Err(CoreError::Parse("expected ')'".into()))}self.consume();Ok(v)}_= >Err(CoreError::Parse("expected value".into()))}}
}
fn parse_function(n:&str)->CoreResult<Function>{Ok(match n{"sin"=>Function::Sin,"cos"=>Function::Cos,"tan"=>Function::Tan,"asin"=>Function::Asin,"acos"=>Function::Acos,"atan"=>Function::Atan,"sqrt"=>Function::Sqrt,"abs"=>Function::Abs,"ln"=>Function::Ln,"log"=>Function::Log,"exp"=>Function::Exp,"floor"=>Function::Floor,"ceil"=>Function::Ceil,_=>return Err(CoreError::Parse(format!("unknown function: {n}")))})}
#[cfg(test)]mod tests{use super::*;#[test]fn arithmetic(){assert_eq!(Expression::eval("1+2*3").unwrap(),7.0)}#[test]fn variable(){assert_eq!(Expression::compile("2x+1").unwrap().eval_x(3.0),7.0)}#[test]fn graph(){assert!(!Expression::graph("sin(x)",GraphConfig::default()).unwrap().is_empty())}}
