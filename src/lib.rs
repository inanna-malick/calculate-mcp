use pest::Parser;
use pest_derive::Parser;
use anyhow::{Result, anyhow};

#[derive(Parser)]
#[grammar = "compute.pest"]
pub struct ComputeParser;

/// Evaluate an arithmetic expression string
pub fn evaluate(expression: &str) -> Result<f64> {
    let mut pairs = ComputeParser::parse(Rule::expr, expression)
        .map_err(|e| anyhow!("Parse error: {}", e))?;
    
    // Get the expr rule
    let expr = pairs.next().unwrap();
    
    // Skip the SOI and get to the actual expression
    let mut inner = expr.into_inner();
    let additive = inner.next().unwrap();
    
    // The additive should be the actual expression
    if additive.as_rule() != Rule::additive {
        return Err(anyhow!("Expected additive rule, got {:?}", additive.as_rule()));
    }
    
    evaluate_additive(additive)
}

fn evaluate_additive(pair: pest::iterators::Pair<Rule>) -> Result<f64> {
    let mut pairs = pair.into_inner();
    let mut result = evaluate_multiplicative(pairs.next().unwrap())?;
    
    // Process any operators and operands
    while let Some(op) = pairs.next() {
        let operand = evaluate_multiplicative(pairs.next().unwrap())?;
        match op.as_str() {
            "+" => result += operand,
            "-" => result -= operand,
            _ => unreachable!("Unexpected operator: {}", op.as_str()),
        }
    }
    
    Ok(result)
}

fn evaluate_multiplicative(pair: pest::iterators::Pair<Rule>) -> Result<f64> {
    let mut pairs = pair.into_inner();
    let mut result = evaluate_primary(pairs.next().unwrap())?;
    
    // Process any operators and operands
    while let Some(op) = pairs.next() {
        let operand = evaluate_primary(pairs.next().unwrap())?;
        match op.as_str() {
            "*" => result *= operand,
            "/" => {
                if operand == 0.0 {
                    return Err(anyhow!("Division by zero"));
                }
                result /= operand;
            }
            _ => unreachable!("Unexpected operator: {}", op.as_str()),
        }
    }
    
    Ok(result)
}

fn evaluate_primary(pair: pest::iterators::Pair<Rule>) -> Result<f64> {
    match pair.as_rule() {
        Rule::number => {
            pair.as_str().parse::<f64>()
                .map_err(|e| anyhow!("Invalid number: {}", e))
        }
        Rule::primary => {
            // Primary can contain either a number or a parenthesized additive
            let inner = pair.into_inner().next().unwrap();
            match inner.as_rule() {
                Rule::number => {
                    inner.as_str().parse::<f64>()
                        .map_err(|e| anyhow!("Invalid number: {}", e))
                }
                Rule::additive => evaluate_additive(inner),
                _ => unreachable!("Unexpected rule in primary: {:?}", inner.as_rule()),
            }
        }
        _ => unreachable!("Unexpected rule: {:?}", pair.as_rule()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;


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
    fn test_division_by_zero() {
        assert!(evaluate("5 / 0").is_err());
    }
}
