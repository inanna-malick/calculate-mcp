use proptest::prelude::*;
use compute_mcp::evaluate;

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
    fn addition_is_commutative(a in -100.0f64..100.0, b in -100.0f64..100.0) {
        let expr1 = format!("{} + {}", a, b);
        let expr2 = format!("{} + {}", b, a);
        let result1 = evaluate(&expr1).unwrap();
        let result2 = evaluate(&expr2).unwrap();
        assert!((result1 - result2).abs() < 0.0001);
    }
    
    #[test]
    fn multiplication_is_commutative(a in -100.0f64..100.0, b in -100.0f64..100.0) {
        let expr1 = format!("{} * {}", a, b);
        let expr2 = format!("{} * {}", b, a);
        let result1 = evaluate(&expr1).unwrap();
        let result2 = evaluate(&expr2).unwrap();
        assert!((result1 - result2).abs() < 0.0001);
    }
    
    #[test]
    fn addition_is_associative(
        a in -50.0f64..50.0, 
        b in -50.0f64..50.0, 
        c in -50.0f64..50.0
    ) {
        let expr1 = format!("({} + {}) + {}", a, b, c);
        let expr2 = format!("{} + ({} + {})", a, b, c);
        let result1 = evaluate(&expr1).unwrap();
        let result2 = evaluate(&expr2).unwrap();
        assert!((result1 - result2).abs() < 0.0001);
    }
    
    #[test]
    fn multiplication_distributes_over_addition(
        a in -20.0f64..20.0,
        b in -20.0f64..20.0,
        c in -20.0f64..20.0
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
    fn division_by_zero_is_error(n in -100.0f64..100.0) {
        let expr = format!("{} / 0", n);
        assert!(evaluate(&expr).is_err());
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
