use pest::Parser;
use pest_derive::Parser;
use std::fmt;
use thiserror::Error;

#[derive(Parser)]
#[grammar = "compute.pest"]
pub struct ComputeParser;

/// AST representation of arithmetic expressions
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Number(f64),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
    Neg(Box<Expr>),
}

impl Expr {
    /// Pretty-print the expression to a pest-grammar-compliant string
    /// Always adds parentheses for clarity
    pub fn to_string(&self) -> String {
        match self {
            Expr::Number(n) => {
                // Format number to avoid scientific notation and trailing zeros
                if n.fract() == 0.0 && n.abs() < 1e15 {
                    format!("{:.0}", n)
                } else {
                    format!("{}", n)
                }
            }
            Expr::Add(left, right) => format!("({} + {})", left.to_string(), right.to_string()),
            Expr::Sub(left, right) => format!("({} - {})", left.to_string(), right.to_string()),
            Expr::Mul(left, right) => format!("({} * {})", left.to_string(), right.to_string()),
            Expr::Div(left, right) => format!("({} / {})", left.to_string(), right.to_string()),
            Expr::Neg(expr) => format!("-{}", expr.to_string()),
        }
    }
}

/// Strong type for arithmetic expressions
#[derive(Debug, Clone, PartialEq)]
pub struct Expression(String);

impl Expression {
    /// Create a new expression, returning None if empty
    pub fn new(expr: impl Into<String>) -> Option<Self> {
        let expr = expr.into();
        if expr.trim().is_empty() {
            None
        } else {
            Some(Self(expr))
        }
    }

    /// Get the expression as a string slice
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

/// Custom error type for compute operations
#[derive(Error, Debug, Clone, PartialEq)]
pub enum ComputeError {
    #[error("Parse error: {0}")]
    ParseError(#[from] pest::error::Error<Rule>),
    
    #[error("Invalid number: {0}")]
    InvalidNumber(#[from] std::num::ParseFloatError),
    
    #[error("Division by zero")]
    DivisionByZero,
    
    #[error("Invalid expression structure: {context}")]
    InvalidStructure { context: String },
    
    #[error("Empty expression")]
    EmptyExpression,
}

pub type Result<T> = std::result::Result<T, ComputeError>;

/// Evaluate an arithmetic expression from a string
pub fn evaluate(expr: &str) -> Result<f64> {
    if expr.trim().is_empty() {
        return Err(ComputeError::EmptyExpression);
    }
    
    let expression = Expression::from(expr);
    evaluate_expression(&expression)
}

/// Parse expression into AST
pub fn parse_expression(expr: &Expression) -> Result<Expr> {
    let mut pairs = ComputeParser::parse(Rule::expr, expr.as_str())?;
    
    let expr_pair = pairs.next().ok_or(ComputeError::InvalidStructure {
        context: "Expected expression but found empty input".to_string()
    })?;
    let mut inner = expr_pair.into_inner();
    let additive = inner.next().ok_or(ComputeError::InvalidStructure {
        context: "Expected additive expression at top level".to_string()
    })?;
    
    parse_additive(additive)
}

fn parse_additive(pair: pest::iterators::Pair<Rule>) -> Result<Expr> {
    let mut pairs = pair.into_inner();
    let mut left = parse_multiplicative(
        pairs.next().ok_or(ComputeError::InvalidStructure {
            context: "Expected left operand in additive expression".to_string()
        })?
    )?;
    
    while let Some(op) = pairs.next() {
        let right = parse_multiplicative(
            pairs.next().ok_or(ComputeError::InvalidStructure {
                context: "Expected right operand after + or - operator".to_string()
            })?
        )?;
        
        left = match op.as_str() {
            "+" => Expr::Add(Box::new(left), Box::new(right)),
            "-" => Expr::Sub(Box::new(left), Box::new(right)),
            _ => return Err(ComputeError::InvalidStructure {
                context: format!("Unknown operator '{}' in additive expression", op.as_str())
            }),
        };
    }
    
    Ok(left)
}

fn parse_multiplicative(pair: pest::iterators::Pair<Rule>) -> Result<Expr> {
    let mut pairs = pair.into_inner();
    let mut left = parse_unary(
        pairs.next().ok_or(ComputeError::InvalidStructure {
            context: "Expected left operand in multiplicative expression".to_string()
        })?
    )?;
    
    while let Some(op) = pairs.next() {
        let right = parse_unary(
            pairs.next().ok_or(ComputeError::InvalidStructure {
                context: "Expected right operand after * or / operator".to_string()
            })?
        )?;
        
        left = match op.as_str() {
            "*" => Expr::Mul(Box::new(left), Box::new(right)),
            "/" => Expr::Div(Box::new(left), Box::new(right)),
            _ => return Err(ComputeError::InvalidStructure {
                context: format!("Unknown operator '{}' in multiplicative expression", op.as_str())
            }),
        };
    }
    
    Ok(left)
}

fn parse_unary(pair: pest::iterators::Pair<Rule>) -> Result<Expr> {
    match pair.as_rule() {
        Rule::unary => {
            let inner_str = pair.as_str();
            if inner_str.starts_with('-') {
                // It's a unary minus
                let mut inner = pair.into_inner();
                // Skip the minus sign token and get the operand
                let operand = inner.next().ok_or(ComputeError::InvalidStructure {
                    context: "Expected operand after unary minus".to_string()
                })?;
                Ok(Expr::Neg(Box::new(parse_unary(operand)?)))
            } else {
                // It's just a primary expression wrapped in unary
                let mut inner = pair.into_inner();
                let first = inner.next().ok_or(ComputeError::InvalidStructure {
                    context: "Expected primary expression in unary".to_string()
                })?;
                parse_primary(first)
            }
        }
        _ => parse_primary(pair),
    }
}

fn parse_primary(pair: pest::iterators::Pair<Rule>) -> Result<Expr> {
    match pair.as_rule() {
        Rule::number => Ok(Expr::Number(pair.as_str().parse()?)),
        Rule::primary => {
            let inner = pair.into_inner()
                .next()
                .ok_or(ComputeError::InvalidStructure {
                    context: "Expected inner expression in parentheses".to_string()
                })?;
            
            match inner.as_rule() {
                Rule::number => Ok(Expr::Number(inner.as_str().parse()?)),
                Rule::additive => parse_additive(inner),
                _ => Err(ComputeError::InvalidStructure {
                    context: format!("Unexpected rule {:?} in primary expression", inner.as_rule())
                }),
            }
        }
        _ => Err(ComputeError::InvalidStructure {
            context: format!("Unexpected rule {:?} where primary expression expected", pair.as_rule())
        }),
    }
}

/// Evaluate an AST expression
pub fn eval_expr(expr: &Expr) -> Result<f64> {
    match expr {
        Expr::Number(n) => Ok(*n),
        Expr::Add(left, right) => Ok(eval_expr(left)? + eval_expr(right)?),
        Expr::Sub(left, right) => Ok(eval_expr(left)? - eval_expr(right)?),
        Expr::Mul(left, right) => Ok(eval_expr(left)? * eval_expr(right)?),
        Expr::Div(left, right) => {
            let divisor = eval_expr(right)?;
            if divisor == 0.0 {
                Err(ComputeError::DivisionByZero)
            } else {
                Ok(eval_expr(left)? / divisor)
            }
        }
        Expr::Neg(expr) => Ok(-eval_expr(expr)?),
    }
}

/// Evaluate a strongly-typed expression
fn evaluate_expression(expr: &Expression) -> Result<f64> {
    let ast = parse_expression(expr)?;
    eval_expr(&ast)
}

/// Batch evaluation result
#[derive(Debug, Clone, PartialEq)]
pub struct EvaluationResult {
    pub expression: Expression,
    pub value: Result<f64>,
}

/// Evaluate multiple expressions at once
pub fn evaluate_batch(expressions: &[Expression]) -> Vec<EvaluationResult> {
    expressions.iter()
        .map(|expr| EvaluationResult {
            expression: expr.clone(),
            value: evaluate_expression(expr),
        })
        .collect()
}

