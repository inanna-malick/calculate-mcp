use pest::Parser;
use pest_derive::Parser;
use std::fmt;

#[derive(Parser)]
#[grammar = "compute.pest"]
pub struct ComputeParser;

/// 🔮 Pure arithmetic expression
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Number(f64),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
    Neg(Box<Expr>),
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Number(n) if n.fract() == 0.0 && n.abs() < 1e15 => write!(f, "{:.0}", n),
            Expr::Number(n) => write!(f, "{}", n),
            Expr::Add(l, r) => write!(f, "({} + {})", l, r),
            Expr::Sub(l, r) => write!(f, "({} - {})", l, r),
            Expr::Mul(l, r) => write!(f, "({} * {})", l, r),
            Expr::Div(l, r) => write!(f, "({} / {})", l, r),
            Expr::Neg(e) => write!(f, "-{}", e),
        }
    }
}

/// 💎 Expression with vibes
#[derive(Debug, Clone, PartialEq)]
pub struct Expression(String);

impl Expression {
    pub fn new(expr: impl Into<String>) -> Option<Self> {
        let expr = expr.into();
        (!expr.trim().is_empty()).then(|| Self(expr))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for Expression {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// 🌊 Crystalline errors
#[derive(Debug, Clone, PartialEq)]
pub enum ComputeError {
    ParseError(Box<pest::error::Error<Rule>>),
    InvalidNumber(std::num::ParseFloatError),
    DivisionByZero,
    InvalidStructure(String),
    EmptyExpression,
}

impl fmt::Display for ComputeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ParseError(e) => write!(f, "Parse: {}", e),
            Self::InvalidNumber(e) => write!(f, "Number: {}", e),
            Self::DivisionByZero => write!(f, "Division by zero"),
            Self::InvalidStructure(ctx) => write!(f, "Structure: {}", ctx),
            Self::EmptyExpression => write!(f, "Empty expression"),
        }
    }
}

impl std::error::Error for ComputeError {}

impl From<Box<pest::error::Error<Rule>>> for ComputeError {
    fn from(e: Box<pest::error::Error<Rule>>) -> Self {
        Self::ParseError(e)
    }
}

impl From<std::num::ParseFloatError> for ComputeError {
    fn from(e: std::num::ParseFloatError) -> Self {
        Self::InvalidNumber(e)
    }
}

pub type Result<T> = std::result::Result<T, ComputeError>;

/// 🎯 Direct evaluation
pub fn evaluate(expr: &str) -> Result<f64> {
    expr.trim().is_empty()
        .then(|| Err(ComputeError::EmptyExpression))
        .unwrap_or_else(|| evaluate_expression(&Expression::from(expr)))
}

/// 🎯 Parse to AST
pub fn parse_expression(expr: &Expression) -> Result<Expr> {
    let mut pairs = ComputeParser::parse(Rule::expr, expr.as_str())
        .map_err(Box::new)?;
    
    pairs.next()
        .and_then(|p| p.into_inner().next())
        .ok_or(ComputeError::InvalidStructure("Empty parse".into()))
        .and_then(parse_additive)
}

fn parse_additive(pair: pest::iterators::Pair<Rule>) -> Result<Expr> {
    let mut pairs = pair.into_inner();
    let mut left = parse_multiplicative(pairs.next().ok_or(
        ComputeError::InvalidStructure("Missing left operand".into())
    )?)?;
    
    while let Some(op) = pairs.next() {
        let right = parse_multiplicative(pairs.next().ok_or(
            ComputeError::InvalidStructure("Missing right operand".into())
        )?)?;
        
        left = match op.as_str() {
            "+" => Expr::Add(Box::new(left), Box::new(right)),
            "-" => Expr::Sub(Box::new(left), Box::new(right)),
            _ => return Err(ComputeError::InvalidStructure(format!("Bad op: {}", op.as_str()))),
        };
    }
    
    Ok(left)
}

fn parse_multiplicative(pair: pest::iterators::Pair<Rule>) -> Result<Expr> {
    let mut pairs = pair.into_inner();
    let mut left = parse_unary(pairs.next().ok_or(
        ComputeError::InvalidStructure("Missing left operand".into())
    )?)?;
    
    while let Some(op) = pairs.next() {
        let right = parse_unary(pairs.next().ok_or(
            ComputeError::InvalidStructure("Missing right operand".into())
        )?)?;
        
        left = match op.as_str() {
            "*" => Expr::Mul(Box::new(left), Box::new(right)),
            "/" => Expr::Div(Box::new(left), Box::new(right)),
            _ => return Err(ComputeError::InvalidStructure(format!("Bad op: {}", op.as_str()))),
        };
    }
    
    Ok(left)
}

fn parse_unary(pair: pest::iterators::Pair<Rule>) -> Result<Expr> {
    match pair.as_rule() {
        Rule::unary if pair.as_str().starts_with('-') => {
            pair.into_inner().next()
                .ok_or(ComputeError::InvalidStructure("Missing operand".into()))
                .and_then(|op| parse_unary(op).map(|e| Expr::Neg(Box::new(e))))
        }
        Rule::unary => {
            pair.into_inner().next()
                .ok_or(ComputeError::InvalidStructure("Missing primary".into()))
                .and_then(parse_primary)
        }
        _ => parse_primary(pair),
    }
}

fn parse_primary(pair: pest::iterators::Pair<Rule>) -> Result<Expr> {
    match pair.as_rule() {
        Rule::number => pair.as_str().parse().map(Expr::Number).map_err(Into::into),
        Rule::primary => pair.into_inner().next()
            .ok_or(ComputeError::InvalidStructure("Empty parens".into()))
            .and_then(|inner| match inner.as_rule() {
                Rule::number => inner.as_str().parse().map(Expr::Number).map_err(Into::into),
                Rule::additive => parse_additive(inner),
                _ => Err(ComputeError::InvalidStructure(format!("Bad rule: {:?}", inner.as_rule()))),
            }),
        _ => Err(ComputeError::InvalidStructure(format!("Bad primary: {:?}", pair.as_rule()))),
    }
}

/// 🔮 Crystalline evaluation
pub fn eval_expr(expr: &Expr) -> Result<f64> {
    match expr {
        Expr::Number(n) => Ok(*n),
        Expr::Add(l, r) => Ok(eval_expr(l)? + eval_expr(r)?),
        Expr::Sub(l, r) => Ok(eval_expr(l)? - eval_expr(r)?),
        Expr::Mul(l, r) => Ok(eval_expr(l)? * eval_expr(r)?),
        Expr::Div(l, r) => {
            let divisor = eval_expr(r)?;
            if divisor != 0.0 {
                Ok(eval_expr(l)? / divisor)
            } else {
                Err(ComputeError::DivisionByZero)
            }
        }
        Expr::Neg(e) => eval_expr(e).map(|n| -n),
    }
}

fn evaluate_expression(expr: &Expression) -> Result<f64> {
    parse_expression(expr).and_then(|ast| eval_expr(&ast))
}

/// 📦 Batch result
#[derive(Debug, Clone, PartialEq)]
pub struct EvaluationResult {
    pub expression: Expression,
    pub value: Result<f64>,
}

/// 🎯 Batch evaluation
pub fn evaluate_batch(expressions: &[Expression]) -> Vec<EvaluationResult> {
    expressions.iter().map(|expr| EvaluationResult {
        expression: expr.clone(),
        value: evaluate_expression(expr),
    }).collect()
}

