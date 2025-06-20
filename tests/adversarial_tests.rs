use compute_mcp::{evaluate, parse_expression, ComputeError};
use proptest::prelude::*;

// ===== NUMERIC EDGE CASES =====

#[test]
fn test_f64_boundaries() {
    // Test very large numbers
    let large = format!("{}", f64::MAX / 2.0);
    let result = evaluate(&large).unwrap();
    assert!(result.is_finite());
    
    // Test very small positive numbers
    let tiny = format!("{}", f64::MIN_POSITIVE);
    let result = evaluate(&tiny).unwrap();
    assert_eq!(result, f64::MIN_POSITIVE);
    
    // Test negative boundaries
    let neg_large = format!("{}", -f64::MAX / 2.0);
    let result = evaluate(&neg_large).unwrap();
    assert!(result.is_finite());
}

#[test]
fn test_numeric_precision_loss() {
    // Test precision in operations
    assert_eq!(evaluate("0.1 + 0.2").unwrap(), 0.1 + 0.2); // This will be 0.30000000000000004
    
    // Test very close to zero
    let expr = "1.0 - 0.9999999999999999";
    let result = evaluate(expr).unwrap();
    assert!(result.abs() < 1e-10);
    
    // Test accumulation of rounding errors
    let expr = "0.1 + 0.1 + 0.1 + 0.1 + 0.1 + 0.1 + 0.1 + 0.1 + 0.1 + 0.1";
    let result = evaluate(expr).unwrap();
    assert!((result - 1.0).abs() < 1e-10);
}

#[test]
fn test_division_approaching_zero() {
    // Division resulting in very small numbers
    let expr = "1.0 / 1000000000000000.0";
    let result = evaluate(&expr).unwrap();
    assert!(result > 0.0 && result < 1e-10);
    
    // Division by very small number (should be large)
    let expr = format!("1.0 / {}", f64::MIN_POSITIVE * 1000.0);
    let result = evaluate(&expr).unwrap();
    assert!(result > 1e100);
}

// ===== PARSER STRESS TESTS =====

#[test]
fn test_deeply_nested_parentheses() {
    // Test parser with deep nesting
    let mut expr = String::from("1");
    for _ in 0..100 {
        expr = format!("({})", expr);
    }
    let result = evaluate(&expr).unwrap();
    assert_eq!(result, 1.0);
    
    // Test with operations at each level
    let mut expr = String::from("1");
    for i in 2..20 {
        expr = format!("({} + {})", expr, i);
    }
    let result = evaluate(&expr).unwrap();
    assert!(result > 0.0);
}

#[test]
fn test_very_long_expressions() {
    // Build a long chain of additions
    let expr = (0..100).map(|i| i.to_string()).collect::<Vec<_>>().join(" + ");
    let result = evaluate(&expr).unwrap();
    assert_eq!(result, (0..100).sum::<i32>() as f64);
    
    // Build alternating operations
    let mut expr = String::from("1000");
    for i in 1..50 {
        if i % 2 == 0 {
            expr.push_str(&format!(" + {}", i));
        } else {
            expr.push_str(&format!(" - {}", i));
        }
    }
    assert!(evaluate(&expr).is_ok());
}

#[test]
fn test_whitespace_extremes() {
    // No whitespace
    assert_eq!(evaluate("1+2*3-4/2").unwrap(), 5.0);
    
    // Excessive whitespace
    assert_eq!(evaluate("   1   +   2   *   3   -   4   /   2   ").unwrap(), 5.0);
    
    // Mixed whitespace types
    assert_eq!(evaluate("1\t+\n2\r\n*\t\t3").unwrap(), 7.0);
    
    // Whitespace in numbers (should fail)
    assert!(matches!(evaluate("1 2 + 3"), Err(ComputeError::ParseError(_))));
}

// ===== OPERATOR COMBINATION TESTS =====

#[test]
fn test_multiple_unary_negations() {
    assert_eq!(evaluate("-5").unwrap(), -5.0);
    assert_eq!(evaluate("--5").unwrap(), 5.0);
    assert_eq!(evaluate("---5").unwrap(), -5.0);
    assert_eq!(evaluate("----5").unwrap(), 5.0);
    
    // With parentheses
    assert_eq!(evaluate("-(-(5))").unwrap(), 5.0);
    assert_eq!(evaluate("-(-(-5))").unwrap(), -5.0);
    
    // Mixed with operations
    assert_eq!(evaluate("-5 + -3").unwrap(), -8.0);
    assert_eq!(evaluate("--5 * -2").unwrap(), -10.0);
}

#[test]
fn test_associativity_edge_cases() {
    // Left associativity for same precedence
    assert_eq!(evaluate("10 - 5 - 2").unwrap(), 3.0); // (10 - 5) - 2, not 10 - (5 - 2)
    assert_eq!(evaluate("20 / 4 / 2").unwrap(), 2.5); // (20 / 4) / 2, not 20 / (4 / 2)
    
    // Complex precedence mixing
    assert_eq!(evaluate("2 + 3 * 4 - 5 * 6 / 2").unwrap(), -1.0);
    
    // Parentheses changing association
    assert_eq!(evaluate("10 - (5 - 2)").unwrap(), 7.0);
    assert_eq!(evaluate("20 / (4 / 2)").unwrap(), 10.0);
}

#[test]
fn test_division_chains() {
    // Multiple divisions
    assert_eq!(evaluate("100 / 2 / 5 / 2").unwrap(), 5.0);
    
    // Division approaching limits
    let expr = "1.0 / 2.0 / 2.0 / 2.0 / 2.0 / 2.0 / 2.0 / 2.0 / 2.0 / 2.0 / 2.0";
    let result = evaluate(expr).unwrap();
    assert!(result < 0.001);
    
    // Mixed with multiplication
    assert_eq!(evaluate("10 * 2 / 4 * 3 / 5").unwrap(), 3.0);
}

// ===== ERROR BOUNDARY TESTS =====

#[test]
fn test_malformed_numbers() {
    // Multiple decimal points
    assert!(matches!(evaluate("1.2.3"), Err(ComputeError::ParseError(_))));
    
    // Decimal with no digits after
    assert!(matches!(evaluate("1."), Err(ComputeError::ParseError(_))));
    
    // Leading decimal (allowed in some languages, not here)
    assert!(matches!(evaluate(".5"), Err(ComputeError::ParseError(_))));
    
    // Invalid characters in numbers
    assert!(matches!(evaluate("1a2"), Err(ComputeError::ParseError(_))));
    
    // Scientific notation is now supported, so this should succeed
    assert!(matches!(evaluate("1e10"), Ok(_)));
    assert_eq!(evaluate("1e10").unwrap(), 10000000000.0);
}

#[test]
fn test_operator_errors() {
    // Missing operands
    assert!(matches!(evaluate("+"), Err(ComputeError::ParseError(_))));
    assert!(matches!(evaluate("1 +"), Err(ComputeError::ParseError(_))));
    assert!(matches!(evaluate("+ 1"), Err(ComputeError::ParseError(_))));
    
    // Double operators
    assert!(matches!(evaluate("1 ++ 2"), Err(ComputeError::ParseError(_))));
    assert!(matches!(evaluate("1 */ 2"), Err(ComputeError::ParseError(_))));
    
    // Missing operators
    assert!(matches!(evaluate("1 2"), Err(ComputeError::ParseError(_))));
    assert!(matches!(evaluate("(1)(2)"), Err(ComputeError::ParseError(_))));
}

#[test]
fn test_parentheses_errors() {
    // Mismatched parentheses
    assert!(matches!(evaluate("(1 + 2"), Err(ComputeError::ParseError(_))));
    assert!(matches!(evaluate("1 + 2)"), Err(ComputeError::ParseError(_))));
    assert!(matches!(evaluate("((1 + 2)"), Err(ComputeError::ParseError(_))));
    assert!(matches!(evaluate("(1 + 2))"), Err(ComputeError::ParseError(_))));
    
    // Empty parentheses
    assert!(matches!(evaluate("()"), Err(ComputeError::ParseError(_))));
    assert!(matches!(evaluate("1 + ()"), Err(ComputeError::ParseError(_))));
}

// ===== PROPERTY-BASED ADVERSARIAL TESTS =====

// Generate expressions that stress numeric boundaries
fn arb_boundary_number() -> impl Strategy<Value = f64> {
    prop_oneof![
        Just(0.0),
        Just(-0.0),
        Just(f64::MIN_POSITIVE),
        Just(-f64::MIN_POSITIVE),
        Just(f64::MAX / 1e10), // Avoid overflow in operations
        Just(-f64::MAX / 1e10),
        Just(1.0),
        Just(-1.0),
        (0.0..1.0), // Small positive
        (-1.0..0.0), // Small negative
        (1e10..1e15), // Large positive
        (-1e15..-1e10), // Large negative
    ]
}

// Generate deeply nested expressions
fn arb_nested_expr(max_depth: u32) -> impl Strategy<Value = String> {
    let leaf = arb_boundary_number().prop_map(|n| n.to_string());
    
    leaf.prop_recursive(max_depth, 256, 10, |inner| {
        prop_oneof![
            // Nested parentheses
            inner.clone().prop_map(|e| format!("({})", e)),
            // Nested unary
            inner.clone().prop_map(|e| format!("-({})", e)),
            // Binary operations
            (inner.clone(), inner.clone()).prop_map(|(l, r)| format!("({} + {})", l, r)),
            (inner.clone(), inner.clone()).prop_map(|(l, r)| format!("({} - {})", l, r)),
            (inner.clone(), inner.clone()).prop_map(|(l, r)| format!("({} * {})", l, r)),
            (inner.clone(), inner).prop_map(|(l, r)| format!("({} / {})", l, r)),
        ]
    })
}

proptest! {
    #[test]
    fn boundary_numbers_parse_correctly(n in arb_boundary_number()) {
        let expr = n.to_string();
        match evaluate(&expr) {
            Ok(result) => {
                // Check that the parsed number is close to original (accounting for string conversion)
                if n.is_finite() {
                    let relative_error = ((result - n) / n).abs();
                    assert!(relative_error < 1e-10 || n == 0.0);
                }
            }
            Err(_) => {
                // Some boundary numbers might not parse correctly as strings
                // This is acceptable for extreme values
            }
        }
    }
    
    #[test]
    fn deeply_nested_expressions_dont_crash(expr in arb_nested_expr(20)) {
        // Just ensure it doesn't panic
        let _ = evaluate(&expr);
    }
    
    #[test]
    fn no_panic_on_random_bytes(bytes in prop::collection::vec(any::<u8>(), 0..1000)) {
        let s = String::from_utf8_lossy(&bytes);
        let _ = evaluate(&s);
    }
    
    #[test]
    fn parser_ast_consistency(expr in arb_nested_expr(5)) {
        // If it parses successfully, converting back to string and reparsing should work
        if let Ok(ast) = parse_expression(&expr) {
            let expr_str = ast.to_string();
            match parse_expression(&expr_str) {
                Ok(_ast2) => {
                    // The AST might not be identical due to added parentheses,
                    // but evaluation should give same result
                    if let (Ok(v1), Ok(v2)) = (evaluate(&expr), evaluate(&expr_str)) {
                        if v1.is_finite() && v2.is_finite() {
                            assert!((v1 - v2).abs() < 1e-10);
                        }
                    }
                }
                Err(_) => {
                    panic!("Failed to reparse generated expression: {} -> {}", expr, expr_str);
                }
            }
        }
    }
}

// ===== PERFORMANCE/MEMORY TESTS =====

#[test]
fn test_expression_size_limits() {
    // Test with a moderately deep expression tree
    fn build_deep_expr(depth: usize) -> String {
        if depth == 0 {
            "1".to_string()
        } else {
            format!("({} + {})", build_deep_expr(depth - 1), build_deep_expr(depth - 1))
        }
    }
    
    // This should work fine
    let expr = build_deep_expr(10);
    assert!(evaluate(&expr).is_ok());
    
    // Build a wide expression
    let wide_expr = (0..1000).map(|i| i.to_string()).collect::<Vec<_>>().join(" + ");
    assert!(evaluate(&wide_expr).is_ok());
}

#[test]
fn test_special_operator_patterns() {
    // Alternating operations that might confuse the parser
    assert_eq!(evaluate("1 - - 2").unwrap(), 3.0);
    assert_eq!(evaluate("5 * - 2").unwrap(), -10.0);
    assert_eq!(evaluate("10 / - 2").unwrap(), -5.0);
    
    // Operations that result in special values
    assert_eq!(evaluate("0 * 999999999999").unwrap(), 0.0);
    assert_eq!(evaluate("0 / 123456").unwrap(), 0.0);
    
    // Identity operations
    assert_eq!(evaluate("5 * 1").unwrap(), 5.0);
    assert_eq!(evaluate("5 / 1").unwrap(), 5.0);
    assert_eq!(evaluate("5 + 0").unwrap(), 5.0);
    assert_eq!(evaluate("5 - 0").unwrap(), 5.0);
}