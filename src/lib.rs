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
    
    #[error("Invalid expression structure")]
    InvalidStructure,
    
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
    
    let expr_pair = pairs.next().ok_or(ComputeError::InvalidStructure)?;
    let mut inner = expr_pair.into_inner();
    let additive = inner.next().ok_or(ComputeError::InvalidStructure)?;
    
    parse_additive(additive)
}

fn parse_additive(pair: pest::iterators::Pair<Rule>) -> Result<Expr> {
    let mut pairs = pair.into_inner();
    let mut left = parse_multiplicative(
        pairs.next().ok_or(ComputeError::InvalidStructure)?
    )?;
    
    while let Some(op) = pairs.next() {
        let right = parse_multiplicative(
            pairs.next().ok_or(ComputeError::InvalidStructure)?
        )?;
        
        left = match op.as_str() {
            "+" => Expr::Add(Box::new(left), Box::new(right)),
            "-" => Expr::Sub(Box::new(left), Box::new(right)),
            _ => return Err(ComputeError::InvalidStructure),
        };
    }
    
    Ok(left)
}

fn parse_multiplicative(pair: pest::iterators::Pair<Rule>) -> Result<Expr> {
    let mut pairs = pair.into_inner();
    let mut left = parse_unary(
        pairs.next().ok_or(ComputeError::InvalidStructure)?
    )?;
    
    while let Some(op) = pairs.next() {
        let right = parse_unary(
            pairs.next().ok_or(ComputeError::InvalidStructure)?
        )?;
        
        left = match op.as_str() {
            "*" => Expr::Mul(Box::new(left), Box::new(right)),
            "/" => Expr::Div(Box::new(left), Box::new(right)),
            _ => return Err(ComputeError::InvalidStructure),
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
                let operand = inner.next().ok_or(ComputeError::InvalidStructure)?;
                Ok(Expr::Neg(Box::new(parse_unary(operand)?)))
            } else {
                // It's just a primary expression wrapped in unary
                let mut inner = pair.into_inner();
                let first = inner.next().ok_or(ComputeError::InvalidStructure)?;
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
                .ok_or(ComputeError::InvalidStructure)?;
            
            match inner.as_rule() {
                Rule::number => Ok(Expr::Number(inner.as_str().parse()?)),
                Rule::additive => parse_additive(inner),
                _ => Err(ComputeError::InvalidStructure),
            }
        }
        _ => Err(ComputeError::InvalidStructure),
    }
}

/// Evaluate an AST expression
fn eval_expr(expr: &Expr) -> Result<f64> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expression_type() {
        assert!(Expression::new("").is_none());
        assert!(Expression::new("  ").is_none());
        assert!(Expression::new("2+3").is_some());
        
        let expr = Expression::from("2 + 3");
        assert_eq!(expr.as_str(), "2 + 3");
        assert_eq!(expr.to_string(), "2 + 3");
    }

    #[test]
    fn test_basic_arithmetic() {
        let expr = Expression::from("42");
        assert_eq!(evaluate_expression(&expr).unwrap(), 42.0);
        
        let expr = Expression::from("2 + 3");
        assert_eq!(evaluate_expression(&expr).unwrap(), 5.0);
        
        let expr = Expression::from("10 - 4");
        assert_eq!(evaluate_expression(&expr).unwrap(), 6.0);
        
        let expr = Expression::from("3 * 4");
        assert_eq!(evaluate_expression(&expr).unwrap(), 12.0);
        
        let expr = Expression::from("15 / 3");
        assert_eq!(evaluate_expression(&expr).unwrap(), 5.0);
    }

    #[test]
    fn test_precedence() {
        let expr = Expression::from("2 + 3 * 4");
        assert_eq!(evaluate_expression(&expr).unwrap(), 14.0);
        
        let expr = Expression::from("10 - 6 / 2");
        assert_eq!(evaluate_expression(&expr).unwrap(), 7.0);
        
        let expr = Expression::from("2 * 3 + 4");
        assert_eq!(evaluate_expression(&expr).unwrap(), 10.0);
    }

    #[test]
    fn test_parentheses() {
        let expr = Expression::from("(2 + 3) * 4");
        assert_eq!(evaluate_expression(&expr).unwrap(), 20.0);
        
        let expr = Expression::from("2 * (3 + 4)");
        assert_eq!(evaluate_expression(&expr).unwrap(), 14.0);
        
        let expr = Expression::from("((1 + 2) * 3) + 4");
        assert_eq!(evaluate_expression(&expr).unwrap(), 13.0);
    }

    #[test]
    fn test_decimals() {
        let expr = Expression::from("3.14");
        assert_eq!(evaluate_expression(&expr).unwrap(), 3.14);
        
        let expr = Expression::from("2.5 * 4");
        assert_eq!(evaluate_expression(&expr).unwrap(), 10.0);
        
        let expr = Expression::from("10 / 4");
        assert_eq!(evaluate_expression(&expr).unwrap(), 2.5);
    }

    #[test]
    fn test_negative_numbers() {
        let expr = Expression::from("-5");
        assert_eq!(evaluate_expression(&expr).unwrap(), -5.0);
        
        let expr = Expression::from("-3 + 5");
        assert_eq!(evaluate_expression(&expr).unwrap(), 2.0);
        
        let expr = Expression::from("10 + -3");
        assert_eq!(evaluate_expression(&expr).unwrap(), 7.0);
    }

    #[test]
    fn test_error_cases() {
        // Division by zero
        let expr = Expression::from("5 / 0");
        matches!(evaluate_expression(&expr), Err(ComputeError::DivisionByZero));
        
        let expr = Expression::from("1 / (2 - 2)");
        matches!(evaluate_expression(&expr), Err(ComputeError::DivisionByZero));
        
        // Empty expression
        assert!(Expression::new("").is_none());
        assert!(Expression::new("   ").is_none());
    }

    #[test]
    fn test_batch_evaluation() {
        let expressions = vec![
            Expression::from("2 + 3"),
            Expression::from("10 / 2"),
            Expression::from("5 / 0"),
            Expression::from("3 * 4"),
        ];
        
        let results = evaluate_batch(&expressions);
        assert_eq!(results.len(), 4);
        assert_eq!(results[0].value.as_ref().unwrap(), &5.0);
        assert_eq!(results[1].value.as_ref().unwrap(), &5.0);
        assert!(matches!(results[2].value, Err(ComputeError::DivisionByZero)));
        assert_eq!(results[3].value.as_ref().unwrap(), &12.0);
    }

    #[test]
    fn test_pretty_printer() {
        // Test simple number
        let expr = parse_expression(&Expression::from("42")).unwrap();
        assert_eq!(expr.to_string(), "42");
        
        // Test addition with parentheses
        let expr = parse_expression(&Expression::from("2 + 3")).unwrap();
        assert_eq!(expr.to_string(), "(2 + 3)");
        
        // Test nested operations
        let expr = parse_expression(&Expression::from("2 + 3 * 4")).unwrap();
        assert_eq!(expr.to_string(), "(2 + (3 * 4))");
        
        // Test complex expression
        let expr = parse_expression(&Expression::from("(2 + 3) * 4")).unwrap();
        assert_eq!(expr.to_string(), "((2 + 3) * 4)");
        
        // Test negation
        let expr = parse_expression(&Expression::from("-5")).unwrap();
        assert_eq!(expr.to_string(), "-5");
        
        // Test negation with expression
        let expr = parse_expression(&Expression::from("-(2 + 3)")).unwrap();
        assert_eq!(expr.to_string(), "-(2 + 3)");
    }
}