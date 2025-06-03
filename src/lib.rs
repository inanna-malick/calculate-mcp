use pest::Parser;
use pest_derive::Parser;
use std::fmt;
use thiserror::Error;

#[derive(Parser)]
#[grammar = "compute.pest"]
pub struct ComputeParser;

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

/// Evaluate an arithmetic expression string
pub fn evaluate(expression: &str) -> Result<f64> {
    let expr = Expression::new(expression)
        .ok_or(ComputeError::EmptyExpression)?;
    evaluate_expression(&expr)
}

/// Evaluate a strongly-typed expression
pub fn evaluate_expression(expr: &Expression) -> Result<f64> {
    let mut pairs = ComputeParser::parse(Rule::expr, expr.as_str())?;
    
    let expr_pair = pairs
        .next()
        .ok_or(ComputeError::InvalidStructure)?;
    
    let mut inner = expr_pair.into_inner();
    let additive = inner
        .next()
        .ok_or(ComputeError::InvalidStructure)?;
    
    // Verify we have the expected rule
    if additive.as_rule() != Rule::additive {
        return Err(ComputeError::InvalidStructure);
    }
    
    evaluate_additive(additive)
}

fn evaluate_additive(pair: pest::iterators::Pair<Rule>) -> Result<f64> {
    let mut pairs = pair.into_inner();
    let mut result = evaluate_multiplicative(
        pairs.next().ok_or(ComputeError::InvalidStructure)?
    )?;
    
    while let Some(op) = pairs.next() {
        let operand = evaluate_multiplicative(
            pairs.next().ok_or(ComputeError::InvalidStructure)?
        )?;
        
        result = match op.as_str() {
            "+" => result + operand,
            "-" => result - operand,
            _ => return Err(ComputeError::InvalidStructure),
        };
    }
    
    Ok(result)
}

fn evaluate_multiplicative(pair: pest::iterators::Pair<Rule>) -> Result<f64> {
    let mut pairs = pair.into_inner();
    let mut result = evaluate_unary(
        pairs.next().ok_or(ComputeError::InvalidStructure)?
    )?;
    
    while let Some(op) = pairs.next() {
        let operand = evaluate_unary(
            pairs.next().ok_or(ComputeError::InvalidStructure)?
        )?;
        
        result = match op.as_str() {
            "*" => result * operand,
            "/" => {
                if operand == 0.0 {
                    return Err(ComputeError::DivisionByZero);
                }
                result / operand
            }
            _ => return Err(ComputeError::InvalidStructure),
        };
    }
    
    Ok(result)
}

fn evaluate_unary(pair: pest::iterators::Pair<Rule>) -> Result<f64> {
    match pair.as_rule() {
        Rule::unary => {
            let inner_str = pair.as_str();
            if inner_str.starts_with('-') {
                // It's a unary minus
                let mut inner = pair.into_inner();
                // Skip the minus sign token and get the operand
                let operand = inner.next().ok_or(ComputeError::InvalidStructure)?;
                Ok(-evaluate_unary(operand)?)
            } else {
                // It's just a primary expression wrapped in unary
                let mut inner = pair.into_inner();
                let first = inner.next().ok_or(ComputeError::InvalidStructure)?;
                evaluate_primary(first)
            }
        }
        _ => evaluate_primary(pair),
    }
}

fn evaluate_primary(pair: pest::iterators::Pair<Rule>) -> Result<f64> {
    match pair.as_rule() {
        Rule::number => Ok(pair.as_str().parse()?),
        Rule::primary => {
            let inner = pair.into_inner()
                .next()
                .ok_or(ComputeError::InvalidStructure)?;
            
            match inner.as_rule() {
                Rule::number => Ok(inner.as_str().parse()?),
                Rule::additive => evaluate_additive(inner),
                _ => Err(ComputeError::InvalidStructure),
            }
        }
        _ => Err(ComputeError::InvalidStructure),
    }
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
        assert_eq!(evaluate("42").unwrap(), 42.0);
        assert_eq!(evaluate("2 + 3").unwrap(), 5.0);
        assert_eq!(evaluate("10 - 4").unwrap(), 6.0);
        assert_eq!(evaluate("3 * 4").unwrap(), 12.0);
        assert_eq!(evaluate("15 / 3").unwrap(), 5.0);
    }

    #[test]
    fn test_precedence() {
        assert_eq!(evaluate("2 + 3 * 4").unwrap(), 14.0);
        assert_eq!(evaluate("10 - 6 / 2").unwrap(), 7.0);
        assert_eq!(evaluate("2 * 3 + 4").unwrap(), 10.0);
    }

    #[test]
    fn test_parentheses() {
        assert_eq!(evaluate("(2 + 3) * 4").unwrap(), 20.0);
        assert_eq!(evaluate("2 * (3 + 4)").unwrap(), 14.0);
        assert_eq!(evaluate("((1 + 2) * 3) + 4").unwrap(), 13.0);
    }

    #[test]
    fn test_decimals() {
        assert_eq!(evaluate("3.14").unwrap(), 3.14);
        assert_eq!(evaluate("2.5 * 4").unwrap(), 10.0);
        assert_eq!(evaluate("10 / 4").unwrap(), 2.5);
    }

    #[test]
    fn test_negative_numbers() {
        assert_eq!(evaluate("-5").unwrap(), -5.0);
        assert_eq!(evaluate("-3 + 5").unwrap(), 2.0);
        assert_eq!(evaluate("10 + -3").unwrap(), 7.0);
    }

    #[test]
    fn test_error_cases() {
        // Division by zero
        matches!(evaluate("5 / 0"), Err(ComputeError::DivisionByZero));
        matches!(evaluate("1 / (2 - 2)"), Err(ComputeError::DivisionByZero));
        
        // Empty expression
        matches!(evaluate(""), Err(ComputeError::EmptyExpression));
        matches!(evaluate("   "), Err(ComputeError::EmptyExpression));
        
        // Invalid syntax
        assert!(evaluate("2 +").is_err());
        assert!(evaluate("+ 2").is_err());
        assert!(evaluate("2 2").is_err());
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
}