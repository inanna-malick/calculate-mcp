use pest::Parser;
use pest_derive::Parser;
use pest::pratt_parser::{Assoc, Op, PrattParser};
use std::fmt;

#[derive(Parser)]
#[grammar = "compute.pest"]
pub struct ComputeParser;

/// Abstract syntax tree for arithmetic expressions
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
            Expr::Number(n) => write!(f, "{}", n),
            Expr::Add(l, r) => write!(f, "({} + {})", l, r),
            Expr::Sub(l, r) => write!(f, "({} - {})", l, r),
            Expr::Mul(l, r) => write!(f, "({} * {})", l, r),
            Expr::Div(l, r) => write!(f, "({} / {})", l, r),
            Expr::Neg(e) => write!(f, "-({})", e),
        }
    }
}

/// Error types for expression evaluation
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
            Self::ParseError(e) => write!(f, "{}", e),
            Self::InvalidNumber(e) => write!(f, "{}", e),
            Self::DivisionByZero => write!(f, "Division by zero"),
            Self::InvalidStructure(msg) => write!(f, "{}", msg),
            Self::EmptyExpression => write!(f, "Empty expression"),
        }
    }
}

impl std::error::Error for ComputeError {}

pub type Result<T> = std::result::Result<T, ComputeError>;

lazy_static::lazy_static! {
    static ref PRATT_PARSER: PrattParser<Rule> = {
        use Assoc::*;
        use Rule::*;

        PrattParser::new()
            .op(Op::infix(add, Left) | Op::infix(subtract, Left))
            .op(Op::infix(multiply, Left) | Op::infix(divide, Left))
            .op(Op::prefix(neg))
    };
}

/// Evaluate an arithmetic expression string
pub fn evaluate(expr: &str) -> Result<f64> {
    let expr = expr.trim();
    if expr.is_empty() {
        return Err(ComputeError::EmptyExpression);
    }
    parse_expression(expr).and_then(|ast| eval_expr(&ast))
}

/// Parse an expression string into an AST using the Pest grammar
pub fn parse_expression(expr: &str) -> Result<Expr> {
    let pairs = ComputeParser::parse(Rule::equation, expr)
        .map_err(|e| ComputeError::ParseError(Box::new(e)))?;
    
    let expr_pair = pairs
        .into_iter()
        .next()
        .ok_or(ComputeError::InvalidStructure("No expression found".into()))?;
    
    parse_expr(expr_pair.into_inner())
}

fn parse_expr(pairs: pest::iterators::Pairs<Rule>) -> Result<Expr> {
    PRATT_PARSER
        .map_primary(|primary| match primary.as_rule() {
            Rule::number => primary
                .as_str()
                .parse()
                .map(Expr::Number)
                .map_err(ComputeError::InvalidNumber),
            Rule::expr => parse_expr(primary.into_inner()),
            Rule::primary => {
                let mut inner = primary.into_inner();
                let mut neg_count = 0;
                
                // Count negation operators
                while let Some(pair) = inner.peek() {
                    if matches!(pair.as_rule(), Rule::neg) {
                        neg_count += 1;
                        inner.next();
                    } else {
                        break;
                    }
                }
                
                // Parse the atom
                let atom = inner.next()
                    .ok_or(ComputeError::InvalidStructure("Missing atom in primary".into()))?;
                
                let mut expr = match atom.as_rule() {
                    Rule::number => atom
                        .as_str()
                        .parse()
                        .map(Expr::Number)
                        .map_err(ComputeError::InvalidNumber)?,
                    Rule::expr => parse_expr(atom.into_inner())?,
                    _ => return Err(ComputeError::InvalidStructure(format!(
                        "Unexpected atom: {:?}",
                        atom.as_rule()
                    ))),
                };
                
                // Apply negations
                for _ in 0..neg_count {
                    expr = Expr::Neg(Box::new(expr));
                }
                
                Ok(expr)
            }
            _ => Err(ComputeError::InvalidStructure(format!(
                "Unexpected primary: {:?}",
                primary.as_rule()
            ))),
        })
        .map_prefix(|op, rhs| match op.as_rule() {
            Rule::neg => Ok(Expr::Neg(Box::new(rhs?))),
            _ => Err(ComputeError::InvalidStructure(format!(
                "Unknown prefix operator: {:?}",
                op.as_rule()
            ))),
        })
        .map_infix(|lhs, op, rhs| match op.as_rule() {
            Rule::add => Ok(Expr::Add(Box::new(lhs?), Box::new(rhs?))),
            Rule::subtract => Ok(Expr::Sub(Box::new(lhs?), Box::new(rhs?))),
            Rule::multiply => Ok(Expr::Mul(Box::new(lhs?), Box::new(rhs?))),
            Rule::divide => Ok(Expr::Div(Box::new(lhs?), Box::new(rhs?))),
            _ => Err(ComputeError::InvalidStructure(format!(
                "Unknown infix operator: {:?}",
                op.as_rule()
            ))),
        })
        .parse(pairs)
}

/// Evaluate an AST expression to produce a numeric result
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

/// Result of evaluating a single expression in a batch
#[derive(Debug, Clone, PartialEq)]
pub struct EvaluationResult {
    pub expression: String,
    pub value: Result<f64>,
}

/// Evaluate multiple expressions in a batch
pub fn evaluate_batch(expressions: &[&str]) -> Vec<EvaluationResult> {
    expressions
        .iter()
        .map(|&expr| EvaluationResult {
            expression: expr.to_string(),
            value: evaluate(expr),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_number() {
        let expr = parse_expression("42").unwrap();
        assert_eq!(expr, Expr::Number(42.0));
        
        let expr = parse_expression("3.14").unwrap();
        assert_eq!(expr, Expr::Number(3.14));
        
        let expr = parse_expression("-10").unwrap();
        assert_eq!(expr, Expr::Neg(Box::new(Expr::Number(10.0))));
    }

    #[test]
    fn test_parse_simple_ops() {
        let expr = parse_expression("2 + 3").unwrap();
        assert_eq!(expr, Expr::Add(
            Box::new(Expr::Number(2.0)),
            Box::new(Expr::Number(3.0))
        ));
        
        let expr = parse_expression("10 - 4").unwrap();
        assert_eq!(expr, Expr::Sub(
            Box::new(Expr::Number(10.0)),
            Box::new(Expr::Number(4.0))
        ));
        
        let expr = parse_expression("3 * 4").unwrap();
        assert_eq!(expr, Expr::Mul(
            Box::new(Expr::Number(3.0)),
            Box::new(Expr::Number(4.0))
        ));
        
        let expr = parse_expression("15 / 3").unwrap();
        assert_eq!(expr, Expr::Div(
            Box::new(Expr::Number(15.0)),
            Box::new(Expr::Number(3.0))
        ));
    }

    #[test]
    fn test_parse_precedence() {
        let expr = parse_expression("2 + 3 * 4").unwrap();
        assert_eq!(expr, Expr::Add(
            Box::new(Expr::Number(2.0)),
            Box::new(Expr::Mul(
                Box::new(Expr::Number(3.0)),
                Box::new(Expr::Number(4.0))
            ))
        ));
    }

    #[test]
    fn test_parse_parentheses() {
        let expr = parse_expression("(2 + 3) * 4").unwrap();
        assert_eq!(expr, Expr::Mul(
            Box::new(Expr::Add(
                Box::new(Expr::Number(2.0)),
                Box::new(Expr::Number(3.0))
            )),
            Box::new(Expr::Number(4.0))
        ));
    }

    #[test]
    fn test_parse_unary_minus() {
        let expr = parse_expression("-(2 + 3)").unwrap();
        assert_eq!(expr, Expr::Neg(
            Box::new(Expr::Add(
                Box::new(Expr::Number(2.0)),
                Box::new(Expr::Number(3.0))
            ))
        ));
    }

    #[test]
    fn test_evaluate_simple() {
        assert_eq!(evaluate("42").unwrap(), 42.0);
        assert_eq!(evaluate("2 + 3").unwrap(), 5.0);
        assert_eq!(evaluate("10 - 4").unwrap(), 6.0);
        assert_eq!(evaluate("3 * 4").unwrap(), 12.0);
        assert_eq!(evaluate("15 / 3").unwrap(), 5.0);
    }

    #[test]
    fn test_evaluate_precedence() {
        assert_eq!(evaluate("2 + 3 * 4").unwrap(), 14.0);
        assert_eq!(evaluate("10 - 2 * 3").unwrap(), 4.0);
        assert_eq!(evaluate("12 / 3 + 2").unwrap(), 6.0);
    }

    #[test]
    fn test_evaluate_parentheses() {
        assert_eq!(evaluate("(2 + 3) * 4").unwrap(), 20.0);
        assert_eq!(evaluate("(10 - 2) * 3").unwrap(), 24.0);
        assert_eq!(evaluate("12 / (3 + 3)").unwrap(), 2.0);
    }

    #[test]
    fn test_evaluate_negative() {
        assert_eq!(evaluate("-5").unwrap(), -5.0);
        assert_eq!(evaluate("-5 + 3").unwrap(), -2.0);
        assert_eq!(evaluate("-(2 + 3)").unwrap(), -5.0);
        assert_eq!(evaluate("-(-5)").unwrap(), 5.0);
    }

    #[test]
    fn test_division_by_zero() {
        assert!(matches!(evaluate("10 / 0"), Err(ComputeError::DivisionByZero)));
        assert!(matches!(evaluate("(5 - 5) / (3 - 3)"), Err(ComputeError::DivisionByZero)));
    }

    #[test]
    fn test_parse_errors() {
        assert!(matches!(evaluate(""), Err(ComputeError::EmptyExpression)));
        assert!(matches!(evaluate("   "), Err(ComputeError::EmptyExpression)));
        assert!(matches!(evaluate("2 +"), Err(ComputeError::ParseError(_))));
        assert!(matches!(evaluate("hello"), Err(ComputeError::ParseError(_))));
        assert!(matches!(evaluate("2 + + 3"), Err(ComputeError::ParseError(_))));
    }

    #[test]
    fn test_complex_expressions() {
        assert_eq!(evaluate("((2 + 3) * 4 - 5) / (6 - 1)").unwrap(), 3.0);
        assert_eq!(evaluate("1 + 2 * 3 + 4 * 5 + 6").unwrap(), 33.0);
        assert_eq!(evaluate("-2 * -3 + -4").unwrap(), 2.0);
    }

    #[test]
    fn test_whitespace_handling() {
        assert_eq!(evaluate("2+3").unwrap(), 5.0);
        assert_eq!(evaluate("2 + 3").unwrap(), 5.0);
        assert_eq!(evaluate("  2  +  3  ").unwrap(), 5.0);
        assert_eq!(evaluate("2\n+\n3").unwrap(), 5.0);
        assert_eq!(evaluate("2\t+\t3").unwrap(), 5.0);
    }

    #[test]
    fn test_decimal_numbers() {
        assert_eq!(evaluate("3.14").unwrap(), 3.14);
        assert_eq!(evaluate("2.5 + 1.5").unwrap(), 4.0);
        assert_eq!(evaluate("10.0 / 4.0").unwrap(), 2.5);
        assert_eq!(evaluate("-3.14").unwrap(), -3.14);
    }
}