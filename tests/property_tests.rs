use proptest::prelude::*;
use compute_mcp::{evaluate, parse_expression, Expression, ComputeError};

// Strategy for generating valid arithmetic expressions
fn arb_expr() -> impl Strategy<Value = String> {
    let leaf = prop_oneof![
        // Positive integers
        (1..1000i32).prop_map(|n| n.to_string()),
        // Negative integers
        (-1000..-1i32).prop_map(|n| n.to_string()),
        // Positive decimals
        (1..1000i32, 0..99u32).prop_map(|(n, d)| format!("{}.{:02}", n, d)),
        // Negative decimals
        (-1000..-1i32, 0..99u32).prop_map(|(n, d)| format!("{}.{:02}", n, d)),
    ];
    
    leaf.prop_recursive(
        3,  // depth
        32, // max nodes
        10, // items per collection
        |inner| {
            prop_oneof![
                // Binary operations
                (inner.clone(), "[+-]", inner.clone())
                    .prop_map(|(l, op, r)| format!("{} {} {}", l, op, r)),
                (inner.clone(), "[*/]", inner.clone())
                    .prop_map(|(l, op, r)| format!("{} {} {}", l, op, r)),
                // Parentheses
                inner.prop_map(|e| format!("({})", e)),
            ]
        },
    )
}

// Strategy for generating numbers that won't cause overflow
fn arb_safe_number() -> impl Strategy<Value = f64> {
    prop_oneof![
        (-100.0..100.0),
        (0.01..0.99),
        (-0.99..-0.01),
    ]
}

proptest! {
    #[test]
    fn parser_doesnt_crash(expr in arb_expr()) {
        // Just verify the parser doesn't panic
        let _ = evaluate(&expr);
    }
    
    #[test]
    fn numbers_parse_correctly(n in -1000.0f64..1000.0) {
        let expr = n.to_string();
        let result = evaluate(&expr).unwrap();
        // Account for floating point precision
        assert!((result - n).abs() < 0.0001);
    }
    
    #[test]
    fn addition_is_commutative(a in arb_safe_number(), b in arb_safe_number()) {
        let expr1 = format!("{} + {}", a, b);
        let expr2 = format!("{} + {}", b, a);
        let result1 = evaluate(&expr1).unwrap();
        let result2 = evaluate(&expr2).unwrap();
        assert!((result1 - result2).abs() < 0.0001);
    }
    
    #[test]
    fn multiplication_is_commutative(a in arb_safe_number(), b in arb_safe_number()) {
        let expr1 = format!("{} * {}", a, b);
        let expr2 = format!("{} * {}", b, a);
        let result1 = evaluate(&expr1).unwrap();
        let result2 = evaluate(&expr2).unwrap();
        assert!((result1 - result2).abs() < 0.0001);
    }
    
    #[test]
    fn addition_is_associative(
        a in arb_safe_number(), 
        b in arb_safe_number(), 
        c in arb_safe_number()
    ) {
        let expr1 = format!("({} + {}) + {}", a, b, c);
        let expr2 = format!("{} + ({} + {})", a, b, c);
        let result1 = evaluate(&expr1).unwrap();
        let result2 = evaluate(&expr2).unwrap();
        assert!((result1 - result2).abs() < 0.0001);
    }
    
    #[test]
    fn multiplication_is_associative(
        a in arb_safe_number(),
        b in arb_safe_number(),
        c in arb_safe_number()
    ) {
        let expr1 = format!("({} * {}) * {}", a, b, c);
        let expr2 = format!("{} * ({} * {})", a, b, c);
        let result1 = evaluate(&expr1).unwrap();
        let result2 = evaluate(&expr2).unwrap();
        assert!((result1 - result2).abs() < 0.0001);
    }
    
    #[test]
    fn multiplication_distributes_over_addition(
        a in arb_safe_number(),
        b in arb_safe_number(),
        c in arb_safe_number()
    ) {
        let expr1 = format!("{} * ({} + {})", a, b, c);
        let expr2 = format!("{} * {} + {} * {}", a, b, a, c);
        let result1 = evaluate(&expr1).unwrap();
        let result2 = evaluate(&expr2).unwrap();
        assert!((result1 - result2).abs() < 0.0001);
    }
    
    #[test]
    fn precedence_is_correct(
        a in 1.0f64..10.0,
        b in 1.0f64..10.0,
        c in 1.0f64..10.0
    ) {
        // a + b * c should equal a + (b * c), not (a + b) * c
        let expr = format!("{} + {} * {}", a, b, c);
        let expected = a + (b * c);
        let result = evaluate(&expr).unwrap();
        assert!((result - expected).abs() < 0.0001);
    }
    
    #[test]
    fn division_by_zero_is_error(n in arb_safe_number()) {
        let expr = format!("{} / 0", n);
        assert!(matches!(evaluate(&expr), Err(ComputeError::DivisionByZero)));
    }
    
    #[test]
    fn zero_identity_for_addition(n in arb_safe_number()) {
        let expr1 = format!("{} + 0", n);
        let expr2 = format!("0 + {}", n);
        let result1 = evaluate(&expr1).unwrap();
        let result2 = evaluate(&expr2).unwrap();
        assert!((result1 - n).abs() < 0.0001);
        assert!((result2 - n).abs() < 0.0001);
    }
    
    #[test]
    fn one_identity_for_multiplication(n in arb_safe_number()) {
        let expr1 = format!("{} * 1", n);
        let expr2 = format!("1 * {}", n);
        let result1 = evaluate(&expr1).unwrap();
        let result2 = evaluate(&expr2).unwrap();
        assert!((result1 - n).abs() < 0.0001);
        assert!((result2 - n).abs() < 0.0001);
    }
    
    #[test]
    fn subtraction_is_addition_of_negative(a in arb_safe_number(), b in arb_safe_number()) {
        let expr1 = format!("{} - {}", a, b);
        let expr2 = format!("{} + -{}", a, b);
        let result1 = evaluate(&expr1).unwrap();
        let result2 = evaluate(&expr2).unwrap();
        assert!((result1 - result2).abs() < 0.0001);
    }
    
    #[test]
    fn double_negative_is_positive(n in 1.0f64..100.0) {
        let expr = format!("--{}", n);
        match evaluate(&expr) {
            Ok(result) => assert!((result - n).abs() < 0.0001),
            Err(_) => {
                // Parser might not support double negatives, which is fine
                // Try with parentheses
                let expr2 = format!("-(-{})", n);
                let result = evaluate(&expr2).unwrap();
                assert!((result - n).abs() < 0.0001);
            }
        }
    }
    
    #[test]
    fn parentheses_preserve_value(n in arb_safe_number()) {
        let expr1 = n.to_string();
        let expr2 = format!("({})", n);
        let expr3 = format!("(({}))", n);
        
        let result1 = evaluate(&expr1).unwrap();
        let result2 = evaluate(&expr2).unwrap();
        let result3 = evaluate(&expr3).unwrap();
        
        assert!((result1 - n).abs() < 0.0001);
        assert!((result2 - n).abs() < 0.0001);
        assert!((result3 - n).abs() < 0.0001);
    }
    
    #[test]
    fn expression_type_safety(_n: u8) {
        // Test Expression type construction
        assert!(Expression::new("").is_none());
        assert!(Expression::new("   ").is_none());
        
        let expr = Expression::new("2 + 3").unwrap();
        assert_eq!(expr.as_str(), "2 + 3");
        assert_eq!(expr.to_string(), "2 + 3");
    }
    
    #[test]
    fn complex_nested_expressions(
        a in 1.0f64..5.0,
        b in 1.0f64..5.0,
        c in 1.0f64..5.0,
        d in 1.0f64..5.0
    ) {
        let expr = format!("(({} + {}) * ({} - {})) / 2", a, b, c, d);
        let expected = ((a + b) * (c - d)) / 2.0;
        let result = evaluate(&expr).unwrap();
        assert!((result - expected).abs() < 0.0001);
    }
    
    #[test]
    fn whitespace_invariance(n1 in arb_safe_number(), n2 in arb_safe_number()) {
        let expressions = vec![
            format!("{}+{}", n1, n2),
            format!("{} + {}", n1, n2),
            format!("{}  +  {}", n1, n2),
            format!("  {} + {}  ", n1, n2),
        ];
        
        let results: Vec<f64> = expressions
            .iter()
            .map(|e| evaluate(e).unwrap())
            .collect();
        
        // All results should be the same
        for i in 1..results.len() {
            assert!((results[i] - results[0]).abs() < 0.0001);
        }
    }
}

// Regression tests for specific cases
#[test]
fn test_complex_expression() {
    let expr = "((2 + 3) * 4 - 5) / (6 - 1)";
    let result = evaluate(expr).unwrap();
    let expected = ((2.0 + 3.0) * 4.0 - 5.0) / (6.0 - 1.0);
    assert!((result - expected).abs() < 0.0001);
}

#[test]
fn test_whitespace_handling() {
    let expressions = vec![
        "2+3",
        "2 + 3",
        "2  +  3",
        "  2 + 3  ",
        "2\n+\n3",
        "2\t+\t3",
    ];
    
    for expr in expressions {
        let result = evaluate(expr).unwrap();
        assert_eq!(result, 5.0);
    }
}

#[test]
fn test_error_types() {
    // Test specific error types
    assert!(matches!(
        evaluate(""),
        Err(ComputeError::EmptyExpression)
    ));
    
    assert!(matches!(
        evaluate("5 / 0"),
        Err(ComputeError::DivisionByZero)
    ));
    
    assert!(matches!(
        evaluate("abc"),
        Err(ComputeError::ParseError(_))
    ));
}

#[test]
fn test_extreme_values() {
    // Test with very small numbers
    assert!((evaluate("0.0001 + 0.0002").unwrap() - 0.0003).abs() < 0.00001);
    
    // Test with very large numbers
    assert!((evaluate("999999 + 1").unwrap() - 1000000.0).abs() < 0.1);
    
    // Test with mixed scales
    assert!((evaluate("1000000 * 0.000001").unwrap() - 1.0).abs() < 0.0001);
}

proptest! {
    #[test]
    fn round_trip_pretty_print(expr in arb_expr()) {
        // Skip if the expression doesn't parse (invalid)
        if let Ok(expr_obj) = Expression::new(&expr).ok_or(()).and_then(|e| parse_expression(&e).map_err(|_| ())) {
            // Convert AST to string
            let pretty_printed = expr_obj.to_string();
            
            // Parse the pretty-printed string
            let reparsed_expr = Expression::from(pretty_printed.as_str());
            let reparsed_ast = parse_expression(&reparsed_expr).expect("Pretty-printed expr should parse");
            
            // Compare the ASTs (they should be equivalent)
            // Note: We compare evaluation results instead of AST structure
            // because pretty-printing might change associativity but preserve value
            let original_value = evaluate(&expr).expect("Original should evaluate");
            let reparsed_value = evaluate(&pretty_printed).expect("Pretty-printed should evaluate");
            
            assert!(
                (original_value - reparsed_value).abs() < 0.0001,
                "Round-trip failed: {} -> {} -> {}, values: {} != {}",
                expr, pretty_printed, reparsed_ast.to_string(), original_value, reparsed_value
            );
        }
    }
}

#[test]
fn test_scientific_notation() {
    // Some parsers might support scientific notation
    match evaluate("1e3") {
        Ok(result) => assert_eq!(result, 1000.0),
        Err(_) => {
            // If not supported, that's okay - it's not in our spec
        }
    }
}