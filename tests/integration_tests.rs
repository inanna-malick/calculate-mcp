#![allow(clippy::approx_constant)]

use compute_mcp::{ComputeError, Expression, evaluate_batch, evaluate};

#[test]
fn test_stdio_mcp_integration() {
    // This test verifies that the MCP server would handle various expressions correctly
    // through the batch evaluation API that stdio_direct.rs uses
    
    let test_cases: Vec<(&str, Result<f64, ComputeError>)> = vec![
        ("2 + 3", Ok(5.0)),
        ("10 * 5", Ok(50.0)),
        ("100 / 0", Err(ComputeError::DivisionByZero)),
        ("(5 + 3) * 2", Ok(16.0)),
        ("3.14159 * 2", Ok(6.28318)),
        ("-10 + 5", Ok(-5.0)),
        ("2 + 3 * 4", Ok(14.0)),
    ];
    
    let expressions: Vec<Expression> = test_cases.iter()
        .map(|(expr, _)| Expression::from(*expr))
        .collect();
    
    let results = evaluate_batch(&expressions);
    
    for ((expr_str, expected), result) in test_cases.iter().zip(results.iter()) {
        match (&result.value, expected) {
            (Ok(actual), Ok(expected_val)) => {
                assert!(
                    (actual - expected_val).abs() < 0.00001,
                    "Expression '{}' expected {} but got {}",
                    expr_str, expected_val, actual
                );
            }
            (Err(ComputeError::DivisionByZero), Err(ComputeError::DivisionByZero)) => {
                // Expected error matches
            }
            (Err(ComputeError::ParseError(_)), Err(ComputeError::ParseError(_))) => {
                // Expected error matches
            }
            _ => panic!("Expression '{}' result mismatch: got {:?}, expected {:?}", 
                       expr_str, result.value, expected),
        }
    }
    
    // Test parse error case separately
    let parse_error_test = vec![Expression::from("invalid expr")];
    let parse_results = evaluate_batch(&parse_error_test);
    assert!(matches!(parse_results[0].value, Err(ComputeError::ParseError(_))));
}

#[test]
fn test_mcp_server_error_handling() {
    // Test that the MCP server properly handles and reports various error conditions
    // These would be returned as JSON error responses in the actual server
    
    // Parse errors that the MCP client might send
    let malformed_expressions = vec![
        Expression::from("2 +"),           // Incomplete
        Expression::from("(2 + 3"),        // Unclosed paren
        Expression::from("2.3.4"),         // Invalid number
        Expression::from("hello + world"), // Invalid identifiers
    ];
    
    let results = evaluate_batch(&malformed_expressions);
    
    // All should be parse errors
    for (i, result) in results.iter().enumerate() {
        assert!(
            matches!(result.value, Err(ComputeError::ParseError(_))),
            "Expression {} should have resulted in parse error",
            i
        );
    }
    
    // Runtime errors
    let runtime_error_expressions = vec![
        Expression::from("1 / 0"),
        Expression::from("100 / (10 - 10)"),
        Expression::from("(5 * 2) / (3 - 3)"),
    ];
    
    let runtime_results = evaluate_batch(&runtime_error_expressions);
    
    // All should be division by zero errors
    for result in runtime_results {
        assert!(matches!(result.value, Err(ComputeError::DivisionByZero)));
    }
}

#[test]
fn test_mcp_expression_features() {
    // Test all features that the MCP server advertises in its capabilities
    
    // Feature: Correct operator precedence
    assert_eq!(evaluate("2 + 3 * 4").unwrap(), 14.0);
    assert_eq!(evaluate("10 / 2 + 3").unwrap(), 8.0);
    
    // Feature: Parentheses for grouping
    assert_eq!(evaluate("(2 + 3) * 4").unwrap(), 20.0);
    assert_eq!(evaluate("2 * (3 + 4)").unwrap(), 14.0);
    
    // Feature: Decimal number support
    assert_eq!(evaluate("3.14159").unwrap(), 3.14159);
    assert_eq!(evaluate("2.5 * 1.5").unwrap(), 3.75);
    
    // Feature: Negative number support
    assert_eq!(evaluate("-42").unwrap(), -42.0);
    assert_eq!(evaluate("-5 + 10").unwrap(), 5.0);
    
    // Feature: Division by zero detection
    assert!(matches!(
        evaluate("10 / 0"),
        Err(ComputeError::DivisionByZero)
    ));
}

#[test]
fn test_complex_real_world_expressions() {
    // Test expressions that might come from real MCP clients
    
    // Financial calculations
    let price_calc = evaluate("(100 * 1.08) - 5").unwrap(); // Price with tax minus discount
    assert!((price_calc - 103.0).abs() < 0.0001);
    
    // Area calculation
    let area = evaluate("3.14159 * 5 * 5").unwrap(); // πr² for radius 5
    assert!((area - 78.53975).abs() < 0.0001);
    
    // Temperature conversion
    let celsius = evaluate("(98.6 - 32) * 5 / 9").unwrap(); // Fahrenheit to Celsius
    assert!((celsius - 37.0).abs() < 0.1);
    
    // Compound calculation
    let compound = evaluate("1000 * (1 + 0.05) * (1 + 0.05)").unwrap(); // 5% interest, 2 years
    assert!((compound - 1102.5).abs() < 0.1);
}