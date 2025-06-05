//! 🔮 Crystalline property tests
//! 
//! Vibes: <🔬🎀💎> - Perfect mathematical properties

use proptest::prelude::*;
use compute_mcp::{evaluate, parse_expression, Expression, ComputeError};

// 🎯 Expression generation strategy
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

// 💎 Safe number strategy
fn arb_safe_number() -> impl Strategy<Value = f64> {
    prop_oneof![
        (-100.0..100.0),
        (0.01..0.99),
        (-0.99..-0.01),
    ]
}

proptest! {
    #[test]
    fn parser_resilience(expr in arb_expr()) {
        let _ = evaluate(&expr); // 💎 Never panics
    }
    
    #[test]
    fn number_identity(n in -1000.0f64..1000.0) {
        // 🔮 Numbers are themselves
        let result = evaluate(&n.to_string()).unwrap();
        assert!((result - n).abs() < 0.0001);
    }
    
    #[test]
    fn addition_commutative(a in arb_safe_number(), b in arb_safe_number()) {
        // ➕ a + b = b + a
        let forward = evaluate(&format!("{} + {}", a, b)).unwrap();
        let reverse = evaluate(&format!("{} + {}", b, a)).unwrap();
        assert!((forward - reverse).abs() < 0.0001);
    }
    
    #[test]
    fn multiplication_commutative(a in arb_safe_number(), b in arb_safe_number()) {
        // ✖️ a * b = b * a
        let forward = evaluate(&format!("{} * {}", a, b)).unwrap();
        let reverse = evaluate(&format!("{} * {}", b, a)).unwrap();
        assert!((forward - reverse).abs() < 0.0001);
    }
    
    #[test]
    fn addition_associative(a in arb_safe_number(), b in arb_safe_number(), c in arb_safe_number()) {
        // ➕ (a + b) + c = a + (b + c)
        let left = evaluate(&format!("({} + {}) + {}", a, b, c)).unwrap();
        let right = evaluate(&format!("{} + ({} + {})", a, b, c)).unwrap();
        assert!((left - right).abs() < 0.0001);
    }
    
    #[test]
    fn multiplication_associative(a in arb_safe_number(), b in arb_safe_number(), c in arb_safe_number()) {
        // ✖️ (a * b) * c = a * (b * c)
        let left = evaluate(&format!("({} * {}) * {}", a, b, c)).unwrap();
        let right = evaluate(&format!("{} * ({} * {})", a, b, c)).unwrap();
        assert!((left - right).abs() < 0.0001);
    }
    
    #[test]
    fn distributive_property(a in arb_safe_number(), b in arb_safe_number(), c in arb_safe_number()) {
        // 🎀 a * (b + c) = a * b + a * c
        let factored = evaluate(&format!("{} * ({} + {})", a, b, c)).unwrap();
        let expanded = evaluate(&format!("{} * {} + {} * {}", a, b, a, c)).unwrap();
        assert!((factored - expanded).abs() < 0.0001);
    }
    
    #[test]
    fn precedence_correct(a in 1.0f64..10.0, b in 1.0f64..10.0, c in 1.0f64..10.0) {
        // 📊 Multiplication before addition
        let result = evaluate(&format!("{} + {} * {}", a, b, c)).unwrap();
        assert!((result - (a + b * c)).abs() < 0.0001);
    }
    
    #[test]
    fn division_by_zero_error(n in arb_safe_number()) {
        // 🚫 Division by zero is an error
        assert!(matches!(evaluate(&format!("{} / 0", n)), Err(ComputeError::DivisionByZero)));
    }
    
    #[test]
    fn zero_identity_addition(n in arb_safe_number()) {
        // 🌟 n + 0 = 0 + n = n
        assert!((evaluate(&format!("{} + 0", n)).unwrap() - n).abs() < 0.0001);
        assert!((evaluate(&format!("0 + {}", n)).unwrap() - n).abs() < 0.0001);
    }
    
    #[test]
    fn one_identity_multiplication(n in arb_safe_number()) {
        // 🎆 n * 1 = 1 * n = n
        assert!((evaluate(&format!("{} * 1", n)).unwrap() - n).abs() < 0.0001);
        assert!((evaluate(&format!("1 * {}", n)).unwrap() - n).abs() < 0.0001);
    }
    
    #[test]
    fn subtraction_is_negative_addition(a in arb_safe_number(), b in arb_safe_number()) {
        // ➖ a - b = a + (-b)
        let sub = evaluate(&format!("{} - {}", a, b)).unwrap();
        let neg_add = evaluate(&format!("{} + -{}", a, b)).unwrap();
        assert!((sub - neg_add).abs() < 0.0001);
    }
    
    #[test]
    fn double_negative_is_positive(n in 1.0f64..100.0) {
        // 🔄 -(-n) = n
        if let Ok(result) = evaluate(&format!("--{}", n)) {
            assert!((result - n).abs() < 0.0001);
        } else {
            let result = evaluate(&format!("-(-{})", n)).unwrap();
            assert!((result - n).abs() < 0.0001);
        }
    }
    
    #[test]
    fn parentheses_transparent(n in arb_safe_number()) {
        // 🎀 (n) = ((n)) = n
        assert!((evaluate(&n.to_string()).unwrap() - n).abs() < 0.0001);
        assert!((evaluate(&format!("({})", n)).unwrap() - n).abs() < 0.0001);
        assert!((evaluate(&format!("(({}))", n)).unwrap() - n).abs() < 0.0001);
    }
    
    #[test]
    fn expression_type_safety(_n: u8) {
        // 💎 Expression type is safe
        assert!(Expression::new("").is_none());
        assert!(Expression::new("   ").is_none());
        assert_eq!(Expression::new("2 + 3").unwrap().as_str(), "2 + 3");
    }
    
    #[test]
    fn complex_nested_expressions(a in 1.0f64..5.0, b in 1.0f64..5.0, c in 1.0f64..5.0, d in 1.0f64..5.0) {
        // 🌌 Deep nesting works correctly
        let result = evaluate(&format!("(({} + {}) * ({} - {})) / 2", a, b, c, d)).unwrap();
        assert!((result - ((a + b) * (c - d)) / 2.0).abs() < 0.0001);
    }
    
    #[test]
    fn whitespace_invariance(n1 in arb_safe_number(), n2 in arb_safe_number()) {
        // 🌫️ Whitespace doesn't matter
        let results: Vec<f64> = vec![
            format!("{}+{}", n1, n2),
            format!("{} + {}", n1, n2),
            format!("{}  +  {}", n1, n2),
            format!("  {} + {}  ", n1, n2),
        ].iter().map(|e| evaluate(e).unwrap()).collect();
        
        results.windows(2).for_each(|w| assert!((w[0] - w[1]).abs() < 0.0001));
    }
    
    #[test]
    fn round_trip_pretty_print(expr in arb_expr()) {
        // 🔄 AST round-trips correctly
        if let Ok(expr_obj) = Expression::new(&expr).ok_or(()).and_then(|e| parse_expression(&e).map_err(|_| ())) {
            let pretty = expr_obj.to_string();
            let original = evaluate(&expr).unwrap();
            let reparsed = evaluate(&pretty).unwrap();
            assert!((original - reparsed).abs() < 0.0001);
        }
    }
}

// 💎 Regression tests
#[test]
fn test_complex_expression() {
    // 🎯 Complex expression evaluation
    let result = evaluate("((2 + 3) * 4 - 5) / (6 - 1)").unwrap();
    assert!((result - 3.0).abs() < 0.0001);
}

#[test]
fn test_whitespace_handling() {
    // 🌫️ Whitespace variations
    vec!["2+3", "2 + 3", "2  +  3", "  2 + 3  ", "2\n+\n3", "2\t+\t3"]
        .iter()
        .for_each(|expr| assert_eq!(evaluate(expr).unwrap(), 5.0));
}

#[test]
fn test_error_types() {
    // 🌊 Error boundary testing
    assert!(matches!(evaluate(""), Err(ComputeError::EmptyExpression)));
    assert!(matches!(evaluate("5 / 0"), Err(ComputeError::DivisionByZero)));
    assert!(matches!(evaluate("abc"), Err(ComputeError::ParseError(_))));
}

#[test]
fn test_extreme_values() {
    // 🌌 Extreme value handling
    assert!((evaluate("0.0001 + 0.0002").unwrap() - 0.0003).abs() < 0.00001);
    assert!((evaluate("999999 + 1").unwrap() - 1000000.0).abs() < 0.1);
    assert!((evaluate("1000000 * 0.000001").unwrap() - 1.0).abs() < 0.0001);
}

#[test]
fn test_scientific_notation() {
    // 🔬 Scientific notation (optional)
    if let Ok(result) = evaluate("1e3") {
        assert_eq!(result, 1000.0);
    } // Not required by spec
}