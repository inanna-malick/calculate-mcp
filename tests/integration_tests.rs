use compute_mcp::evaluate;

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
    // Test cases that should produce errors
    let error_cases = vec![
        "5 / 0",
        "10 / (5 - 5)",
        "",
        "2 +",
        "+ 3",
        "2 + + 3",
        "(2 + 3",
        "2 + 3)",
        "2 + (3 + )",
    ];
    
    for expr in error_cases {
        assert!(
            evaluate(expr).is_err(),
            "Expression '{}' should have failed but didn't",
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
}
