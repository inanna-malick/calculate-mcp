use compute_mcp::{evaluate, ComputeError, Expression, evaluate_batch};

#[test]
fn test_integration_basic_expressions() {
    // Test various expression types to ensure integration works
    let test_cases = vec![
        ("42", 42.0),
        ("3.14159", 3.14159),
        ("-10", -10.0),
        ("2 + 2", 4.0),
        ("10 - 3", 7.0),
        ("4 * 5", 20.0),
        ("15 / 3", 5.0),
        ("2 + 3 * 4", 14.0),
        ("(2 + 3) * 4", 20.0),
        ("100 / 10 / 2", 5.0),
        ("2 * 3 * 4", 24.0),
        ("10 - 5 - 3", 2.0),
        ("-5 + 10", 5.0),
        ("10 + -5", 5.0),
        ("-10 * -2", 20.0),
        ("-(5 + 3)", -8.0),
        ("3.14 * 2", 6.28),
        ("10 / 4", 2.5),
    ];
    
    for (expr, expected) in test_cases {
        let result = evaluate(expr).expect(&format!("Failed to evaluate: {}", expr));
        assert!(
            (result - expected).abs() < 0.0001,
            "Expression '{}' evaluated to {} but expected {}",
            expr,
            result,
            expected
        );
    }
}

#[test]
fn test_error_cases() {
    // Test specific error types
    assert!(matches!(evaluate(""), Err(ComputeError::EmptyExpression)));
    assert!(matches!(evaluate("   "), Err(ComputeError::EmptyExpression)));
    
    assert!(matches!(evaluate("5 / 0"), Err(ComputeError::DivisionByZero)));
    assert!(matches!(evaluate("10 / (5 - 5)"), Err(ComputeError::DivisionByZero)));
    
    // Test parse errors
    let parse_error_cases = vec![
        "2 +",
        "+ 3",
        "2 + + 3",
        "(2 + 3",
        "2 + 3)",
        "2 + (3 + )",
        "abc",
        "2 + abc",
        "2.3.4",
        "2 2",
        "2 */ 3",
    ];
    
    for expr in parse_error_cases {
        assert!(
            matches!(evaluate(expr), Err(ComputeError::ParseError(_))),
            "Expression '{}' should have resulted in a parse error",
            expr
        );
    }
}

#[test]
fn test_precedence_and_associativity() {
    // Test that operators have correct precedence and associativity
    assert_eq!(evaluate("2 + 3 + 4").unwrap(), 9.0);
    assert_eq!(evaluate("2 * 3 + 4").unwrap(), 10.0);
    assert_eq!(evaluate("2 + 3 * 4").unwrap(), 14.0);
    assert_eq!(evaluate("2 * 3 * 4").unwrap(), 24.0);
    assert_eq!(evaluate("20 / 4 / 2").unwrap(), 2.5); // Left associative
    assert_eq!(evaluate("20 - 10 - 5").unwrap(), 5.0); // Left associative
    
    // Complex precedence tests
    assert_eq!(evaluate("2 + 3 * 4 - 5").unwrap(), 9.0); // 2 + 12 - 5 = 9
    assert_eq!(evaluate("10 / 2 + 3 * 4").unwrap(), 17.0); // 5 + 12 = 17
    assert_eq!(evaluate("10 / (2 + 3) * 4").unwrap(), 8.0); // 10 / 5 * 4 = 8
}

#[test]
fn test_batch_evaluation_integration() {
    let expressions = vec![
        Expression::from("2 + 3"),
        Expression::from("10 * 5"),
        Expression::from("100 / 0"), // Error case
        Expression::from("(5 + 3) * 2"),
        Expression::from("3.14 * 2"),
    ];
    
    let results = evaluate_batch(&expressions);
    
    assert_eq!(results.len(), 5);
    assert_eq!(results[0].value.as_ref().unwrap(), &5.0);
    assert_eq!(results[1].value.as_ref().unwrap(), &50.0);
    assert!(matches!(results[2].value, Err(ComputeError::DivisionByZero)));
    assert_eq!(results[3].value.as_ref().unwrap(), &16.0);
    assert!((results[4].value.as_ref().unwrap() - 6.28).abs() < 0.0001);
    
    // Verify expressions are preserved
    assert_eq!(results[0].expression.as_str(), "2 + 3");
    assert_eq!(results[1].expression.as_str(), "10 * 5");
    assert_eq!(results[2].expression.as_str(), "100 / 0");
}

#[test]
fn test_expression_type_integration() {
    // Test Expression type behavior
    assert!(Expression::new("").is_none());
    assert!(Expression::new("   \t\n  ").is_none());
    
    let expr = Expression::new(" 2 + 3 ").unwrap();
    assert_eq!(expr.as_str(), " 2 + 3 ");
    
    // Test From trait
    let expr2 = Expression::from("10 * 20");
    assert_eq!(expr2.to_string(), "10 * 20");
    
    // Test Display trait
    let display = format!("{}", expr2);
    assert_eq!(display, "10 * 20");
}

#[test]
fn test_complex_nested_expressions() {
    let test_cases = vec![
        ("((2 + 3) * 4 - 5) / (6 - 1)", 3.0), // ((5 * 4) - 5) / 5 = 15 / 5 = 3
        ("(10 + 20) * 30", 900.0),
        ("100 / (10 / 2)", 20.0), // 100 / 5 = 20
        ("2 * (3 + (4 * 5))", 46.0), // 2 * (3 + 20) = 2 * 23 = 46
        ("((((1))))", 1.0), // Multiple nested parentheses
    ];
    
    for (expr, expected) in test_cases {
        let result = evaluate(expr).unwrap();
        assert!(
            (result - expected).abs() < 0.0001,
            "Expression '{}' evaluated to {} but expected {}",
            expr,
            result,
            expected
        );
    }
}