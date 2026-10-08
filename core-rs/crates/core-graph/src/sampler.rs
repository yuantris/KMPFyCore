use core_math::Expr;
#[derive(Debug,Clone,Copy,PartialEq)]
pub struct GraphConfig { pub min_x:f64,pub max_x:f64,pub min_y:f64,pub max_y:f64,pub samples:usize,pub pixel_width:u32,pub pixel_height:u32,pub max_subdivision:u32,pub pixel_error:f64,pub max_screen_jump:f64 }
impl Default for GraphConfig { fn default()->Self{Self{min_x:-10.0,max_x:10.0,min_y:-10.0,max_y:10.0,samples:1200,pixel_width:1200,pixel_height:800,max_subdivision:12,pixel_error:0.75,max_screen_jump:48.0}} }
impl GraphConfig { pub fn validate(&self)->Result<(),&'static str>{ if !self.min_x.is_finite()||!self.max_x.is_finite()||self.min_x>=self.max_x{return Err("invalid x range")} if !self.min_y.is_finite()||!self.max_y.is_finite()||self.min_y>=self.max_y{return Err("invalid y range")} if self.samples<2{return Err("samples must be >= 2")} if self.pixel_width<2||self.pixel_height<2{return Err("pixel size must be >= 2")} if self.max_subdivision==0{return Err("max_subdivision must be > 0")} if !self.pixel_error.is_finite()||self.pixel_error<=0.0{return Err("pixel_error must be > 0")} if !self.max_screen_jump.is_finite()||self.max_screen_jump<=0.0{return Err("max_screen_jump must be > 0")} Ok(()) } }
#[derive(Debug,Clone,Copy,PartialEq)] pub struct GraphPoint{pub x:f64,pub y:f64}
#[derive(Debug,Clone,PartialEq)] pub struct GraphSegment{pub points:Vec<GraphPoint>}
#[derive(Debug,Clone,PartialEq)] pub struct GraphResult{pub segments:Vec<GraphSegment>,pub evaluated_points:usize,pub discontinuities:usize}
pub struct GraphSampler;
impl GraphSampler { pub fn sample(expr:&Expr,cfg:&GraphConfig)->GraphResult{
    if cfg.validate().is_err(){return GraphResult{segments:Vec::new(),evaluated_points:0,discontinuities:0};}
    let mut s=State{expr,cfg,evaluated_points:0,discontinuities:0}; let step=(cfg.max_x-cfg.min_x)/(cfg.samples-1) as f64; let mut segments=Vec::new(); let mut current=Vec::new(); let mut prev=None;
    for i in 0..cfg.samples { let x=if i+1==cfg.samples{cfg.max_x}else{cfg.min_x+step*i as f64}; let p=s.eval(x); match (prev,p){
        (None,Some(p))=>{current.push(p);prev=Some(p)}
        (Some(a),Some(b))=>{let mut mids=Vec::new(); if s.refine(a,b,0,&mut mids){for m in mids{push_unique(&mut current,m)} push_unique(&mut current,b);prev=Some(b)}else{flush(&mut segments,&mut current);s.discontinuities+=1;current.clear();current.push(b);prev=Some(b)}}
        (Some(_),None)=>{flush(&mut segments,&mut current);s.discontinuities+=1;prev=None}
        (None,None)=>{}
    }}
    flush(&mut segments,&mut current); GraphResult{segments,evaluated_points:s.evaluated_points,discontinuities:s.discontinuities}
} }
pub fn sample(expr:&Expr,cfg:&GraphConfig)->GraphResult{GraphSampler::sample(expr,cfg)}
struct State<'a>{expr:&'a Expr,cfg:&'a GraphConfig,evaluated_points:usize,discontinuities:usize}
impl<'a> State<'a>{
 fn eval(&mut self,x:f64)->Option<GraphPoint>{self.evaluated_points+=1;let y=self.expr.eval_x(x);if y.is_finite(){Some(GraphPoint{x,y})}else{None}}
 fn refine(&mut self,a:GraphPoint,b:GraphPoint,depth:u32,mids:&mut Vec<GraphPoint>)->bool{
    let mx=a.x+(b.x-a.x)*0.5;if mx==a.x||mx==b.x{return self.safe(a,b)} let Some(m)=self.eval(mx)else{return false};
    let (_,ay)=self.screen(a);let (_,my)=self.screen(m);let (_,by)=self.screen(b);let err=(my-(ay+by)*0.5).abs();let jump=(by-ay).abs();
    if err<=self.cfg.pixel_error&&jump<=self.cfg.max_screen_jump{return true}
    if depth>=self.cfg.max_subdivision{return err<=self.cfg.pixel_error}
    let mut lm=Vec::new();if !self.refine(a,m,depth+1,&mut lm){return false} let mut rm=Vec::new();if !self.refine(m,b,depth+1,&mut rm){return false} mids.extend(lm);mids.push(m);mids.extend(rm);true
 }
 fn safe(&self,a:GraphPoint,b:GraphPoint)->bool{let(_,ay)=self.screen(a);let(_,by)=self.screen(b);(by-ay).abs()<=self.cfg.max_screen_jump}
 fn screen(&self,p:GraphPoint)->(f64,f64){let xs=self.cfg.max_x-self.cfg.min_x;let ys=self.cfg.max_y-self.cfg.min_y;((p.x-self.cfg.min_x)/xs*(self.cfg.pixel_width-1)as f64,(self.cfg.max_y-p.y)/ys*(self.cfg.pixel_height-1)as f64)}
}
fn push_unique(v:&mut Vec<GraphPoint>,p:GraphPoint){if v.last().map(|q|q.x==p.x).unwrap_or(false){return}v.push(p)}
fn flush(segments:&mut Vec<GraphSegment>,points:&mut Vec<GraphPoint>){if points.len()>=2{segments.push(GraphSegment{points:std::mem::take(points)})}else{points.clear()}}
#[cfg(test)]
mod tests{use super::*;use core_math::Expression;fn cfg()->GraphConfig{GraphConfig{min_x:-4.0,max_x:4.0,min_y:-20.0,max_y:20.0,samples:101,pixel_width:800,pixel_height:600,max_subdivision:14,pixel_error:0.75,max_screen_jump:48.0}}
#[test]fn reciprocal_splits(){let r=sample(&Expression::compile("1/x").unwrap(),&cfg());assert!(r.segments.len()>=2);for s in &r.segments{assert!(!(s.points.first().unwrap().x<0.0&&s.points.last().unwrap().x>0.0));}}
#[test]fn x2_connected(){let r=sample(&Expression::compile("x^2").unwrap(),&cfg());assert_eq!(r.segments.len(),1)}
#[test]fn sqrt_domain(){let r=sample(&Expression::compile("sqrt(x)").unwrap(),&cfg());assert!(!r.segments.is_empty());assert!(r.segments[0].points[0].x>=0.0)}
#[test]fn sin_connected(){let r=sample(&Expression::compile("sin(x)").unwrap(),&cfg());assert_eq!(r.segments.len(),1)}
#[test]fn tan_splits(){let r=sample(&Expression::compile("tan(x)").unwrap(),&cfg());assert!(r.segments.len()>1)} }
