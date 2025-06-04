#![allow(clippy::approx_constant)]

use compute_mcp::{evaluate, ComputeError};

#[test]
fn test_basic_arithmetic() {
    // Numbers
    assert_eq!(evaluate("42").unwrap(), 42.0);
    assert_eq!(evaluate("3.14").unwrap(), 3.14);
    assert_eq!(evaluate("-10").unwrap(), -10.0);
    assert_eq!(evaluate("0").unwrap(), 0.0);
    
    // Addition
    assert_eq!(evaluate("2 + 3").unwrap(), 5.0);
    assert_eq!(evaluate("1.5 + 2.5").unwrap(), 4.0);
    assert_eq!(evaluate("-5 + 10").unwrap(), 5.0);
    assert_eq!(evaluate("10 + -5").unwrap(), 5.0);
    
    // Subtraction
    assert_eq!(evaluate("10 - 4").unwrap(), 6.0);
    assert_eq!(evaluate("5.5 - 2.5").unwrap(), 3.0);
    assert_eq!(evaluate("-5 - 5").unwrap(), -10.0);
    assert_eq!(evaluate("0 - 10").unwrap(), -10.0);
    
    // Multiplication
    assert_eq!(evaluate("3 * 4").unwrap(), 12.0);
    assert_eq!(evaluate("2.5 * 4").unwrap(), 10.0);
    assert_eq!(evaluate("-3 * 4").unwrap(), -12.0);
    assert_eq!(evaluate("-3 * -4").unwrap(), 12.0);
    
    // Division
    assert_eq!(evaluate("15 / 3").unwrap(), 5.0);
    assert_eq!(evaluate("10 / 4").unwrap(), 2.5);
    assert_eq!(evaluate("-20 / 4").unwrap(), -5.0);
    assert_eq!(evaluate("-20 / -4").unwrap(), 5.0);
}

#[test]
fn test_operator_precedence() {
    // Multiplication before addition
    assert_eq!(evaluate("2 + 3 * 4").unwrap(), 14.0);
    assert_eq!(evaluate("3 * 4 + 2").unwrap(), 14.0);
    
    // Division before subtraction
    assert_eq!(evaluate("10 - 6 / 2").unwrap(), 7.0);
    assert_eq!(evaluate("6 / 2 - 1").unwrap(), 2.0);
    
    // Mixed operations
    assert_eq!(evaluate("2 * 3 + 4 * 5").unwrap(), 26.0);
    assert_eq!(evaluate("10 / 2 - 3 * 1").unwrap(), 2.0);
    assert_eq!(evaluate("2 + 3 * 4 - 5").unwrap(), 9.0);
}

#[test]
fn test_parentheses() {
    // Basic parentheses
    assert_eq!(evaluate("(2 + 3) * 4").unwrap(), 20.0);
    assert_eq!(evaluate("2 * (3 + 4)").unwrap(), 14.0);
    assert_eq!(evaluate("(10 - 6) / 2").unwrap(), 2.0);
    
    // Nested parentheses
    assert_eq!(evaluate("((1 + 2) * 3) + 4").unwrap(), 13.0);
    assert_eq!(evaluate("2 * (3 + (4 * 5))").unwrap(), 46.0);
    assert_eq!(evaluate("((10))").unwrap(), 10.0);
    assert_eq!(evaluate("((((1))))").unwrap(), 1.0);
    
    // Complex expressions
    assert_eq!(evaluate("((2 + 3) * 4 - 5) / (6 - 1)").unwrap(), 3.0);
    assert_eq!(evaluate("100 / (10 / 2)").unwrap(), 20.0);
}

#[test]
fn test_associativity() {
    // Left-to-right for same precedence
    assert_eq!(evaluate("20 - 10 - 5").unwrap(), 5.0);
    assert_eq!(evaluate("100 / 10 / 2").unwrap(), 5.0);
    assert_eq!(evaluate("2 + 3 + 4").unwrap(), 9.0);
    assert_eq!(evaluate("2 * 3 * 4").unwrap(), 24.0);
}

#[test]
fn test_unary_negation() {
    // Simple negation
    assert_eq!(evaluate("-5").unwrap(), -5.0);
    assert_eq!(evaluate("-0").unwrap(), 0.0);
    assert_eq!(evaluate("--5").unwrap(), 5.0);
    
    // Negation with expressions
    assert_eq!(evaluate("-(2 + 3)").unwrap(), -5.0);
    assert_eq!(evaluate("-(10 - 15)").unwrap(), 5.0);
    assert_eq!(evaluate("-(-10)").unwrap(), 10.0);
    
    // Negation in complex expressions
    assert_eq!(evaluate("-5 + 10").unwrap(), 5.0);
    assert_eq!(evaluate("10 + -5").unwrap(), 5.0);
    assert_eq!(evaluate("-2 * -3").unwrap(), 6.0);
}

#[test]
fn test_decimal_arithmetic() {
    // Basic decimals
    assert_eq!(evaluate("3.14159").unwrap(), 3.14159);
    assert_eq!(evaluate("0.1 + 0.2").unwrap(), 0.30000000000000004); // Floating point precision
    
    // Decimal operations
    assert_eq!(evaluate("2.5 * 4").unwrap(), 10.0);
    assert_eq!(evaluate("10.5 / 2.1").unwrap(), 5.0);
    assert_eq!(evaluate("3.14 * 2").unwrap(), 6.28);
    
    // Very small numbers
    assert!((evaluate("0.0001 + 0.0002").unwrap() - 0.0003).abs() < 0.00001);
}

#[test]
fn test_edge_cases() {
    // Zero operations
    assert_eq!(evaluate("0 + 0").unwrap(), 0.0);
    assert_eq!(evaluate("0 * 100").unwrap(), 0.0);
    assert_eq!(evaluate("0 / 100").unwrap(), 0.0);
    
    // Identity operations
    assert_eq!(evaluate("5 + 0").unwrap(), 5.0);
    assert_eq!(evaluate("5 * 1").unwrap(), 5.0);
    assert_eq!(evaluate("5 / 1").unwrap(), 5.0);
    
    // Large numbers
    assert_eq!(evaluate("1000000 + 1").unwrap(), 1000001.0);
    assert_eq!(evaluate("1000000 * 0.000001").unwrap(), 1.0);
}

#[test]
fn test_whitespace_handling() {
    // Various whitespace
    assert_eq!(evaluate("2+3").unwrap(), 5.0);
    assert_eq!(evaluate("2 + 3").unwrap(), 5.0);
    assert_eq!(evaluate("  2  +  3  ").unwrap(), 5.0);
    assert_eq!(evaluate("2\t+\t3").unwrap(), 5.0);
    assert_eq!(evaluate("2\n+\n3").unwrap(), 5.0);
}

#[test]
fn test_error_division_by_zero() {
    // Direct division by zero
    assert!(matches!(
        evaluate("5 / 0"),
        Err(ComputeError::DivisionByZero)
    ));
    
    assert!(matches!(
        evaluate("10 / 0.0"),
        Err(ComputeError::DivisionByZero)
    ));
    
    // Expression evaluating to zero
    assert!(matches!(
        evaluate("1 / (2 - 2)"),
        Err(ComputeError::DivisionByZero)
    ));
    
    assert!(matches!(
        evaluate("100 / (10 * 0)"),
        Err(ComputeError::DivisionByZero)
    ));
}

#[test]
fn test_error_empty_expression() {
    assert!(matches!(
        evaluate(""),
        Err(ComputeError::EmptyExpression)
    ));
    
    assert!(matches!(
        evaluate("   "),
        Err(ComputeError::EmptyExpression)
    ));
    
    assert!(matches!(
        evaluate("\t\n"),
        Err(ComputeError::EmptyExpression)
    ));
}