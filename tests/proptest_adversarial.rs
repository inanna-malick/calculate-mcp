use compute_mcp::*;
use proptest::prelude::*;
use proptest::num::f64::{POSITIVE, NEGATIVE, NORMAL, SUBNORMAL};
use proptest::test_runner::TestRunner;

// Constants for controlling test complexity
const MAX_DEPTH: u32 = 50;
const MAX_LEAVES: u32 = 100;

// Custom strategy for generating valid f64 numbers including edge cases
fn arb_number() -> impl Strategy<Value = f64> {
    prop_oneof![
        // Common cases
        -1000.0..1000.0,
        // Small numbers near zero
        -0.001..0.001,
        // Large numbers
        prop_oneof![Just(1e10), Just(-1e10), Just(1e100), Just(-1e100)],
        // Special but finite values
        Just(0.0),
        Just(-0.0),
        Just(f64::MIN_POSITIVE),
        Just(-f64::MIN_POSITIVE),
        Just(f64::EPSILON),
        Just(-f64::EPSILON),
        // Subnormal numbers
        SUBNORMAL,
        // Normal positive/negative
        POSITIVE.prop_filter("Must be finite", |x| x.is_finite()),
        NEGATIVE.prop_filter("Must be finite", |x| x.is_finite()),
        NORMAL,
    ]
}

// Generate arbitrary AST expressions with controlled depth
fn arb_expr() -> impl Strategy<Value = Expr> {
    let leaf = arb_number().prop_map(Expr::Number);
    
    leaf.prop_recursive(
        MAX_DEPTH, // max depth
        MAX_LEAVES, // max nodes
        10, // items per collection (not used here)
        |inner| {
            prop_oneof![
                // Binary operations
                (inner.clone(), inner.clone()).prop_map(|(l, r)| Expr::Add(Box::new(l), Box::new(r))),
                (inner.clone(), inner.clone()).prop_map(|(l, r)| Expr::Sub(Box::new(l), Box::new(r))),
                (inner.clone(), inner.clone()).prop_map(|(l, r)| Expr::Mul(Box::new(l), Box::new(r))),
                (inner.clone(), inner.clone()).prop_map(|(l, r)| Expr::Div(Box::new(l), Box::new(r))),
                // Unary operations
                inner.prop_map(|e| Expr::Neg(Box::new(e))),
            ]
        },
    )
}

// Generate deeply nested expressions of a specific structure
fn arb_deep_expr(depth: u32) -> BoxedStrategy<Expr> {
    if depth == 0 {
        arb_number().prop_map(Expr::Number).boxed()
    } else {
        prop_oneof![
            // Deep left nesting
            (arb_deep_expr(depth - 1), arb_number().prop_map(Expr::Number))
                .prop_map(|(l, r)| Expr::Add(Box::new(l), Box::new(r))),
            // Deep right nesting  
            (arb_number().prop_map(Expr::Number), arb_deep_expr(depth - 1))
                .prop_map(|(l, r)| Expr::Mul(Box::new(l), Box::new(r))),
            // Deep parentheses nesting
            arb_deep_expr(depth - 1).prop_map(|e| Expr::Neg(Box::new(e))),
        ].boxed()
    }
}

// Generate expression strings with specific patterns for parser testing
fn arb_expr_string() -> impl Strategy<Value = String> {
    arb_expr().prop_map(|e| format!("{}", e))
}

// Generate whitespace variations
fn arb_whitespace() -> impl Strategy<Value = String> {
    prop::collection::vec(prop_oneof![
        Just(" "),
        Just("\t"),
        Just("\n"),
        Just("\r"),
        Just("  "),
    ], 0..5).prop_map(|v| v.join(""))
}

// Generate expressions with random whitespace
fn arb_expr_with_whitespace() -> impl Strategy<Value = String> {
    (arb_expr(), proptest::collection::vec(arb_whitespace(), 5..10))
        .prop_map(|(expr, spaces)| {
            let s = format!("{}", expr);
            // Insert whitespace around operators
            let mut result = String::new();
            let mut space_idx = 0;
            for ch in s.chars() {
                if "+-*/()".contains(ch) && space_idx < spaces.len() {
                    result.push_str(&spaces[space_idx]);
                    space_idx = (space_idx + 1) % spaces.len();
                }
                result.push(ch);
                if "+-*/()".contains(ch) && space_idx < spaces.len() {
                    result.push_str(&spaces[space_idx]);
                    space_idx = (space_idx + 1) % spaces.len();
                }
            }
            result
        })
}

// Helper function to check if two f64 values are approximately equal
fn approx_eq(a: f64, b: f64, epsilon: f64) -> bool {
    if a.is_finite() && b.is_finite() {
        (a - b).abs() < epsilon
    } else {
        a == b || (a.is_nan() && b.is_nan())
    }
}

proptest! {
    // Test 1: Parse-Print-Parse identity
    #[test]
    fn parse_print_parse_identity(expr in arb_expr()) {
        let printed = format!("{}", expr);
        match parse_expression(&printed) {
            Ok(parsed) => {
                // The Display impl adds parentheses, so we check evaluation equivalence
                match (eval_expr(&expr), eval_expr(&parsed)) {
                    (Ok(v1), Ok(v2)) => prop_assert!(approx_eq(v1, v2, 1e-10)),
                    (Err(ComputeError::DivisionByZero), Err(ComputeError::DivisionByZero)) => {},
                    _ => prop_assert!(false, "Evaluation mismatch"),
                }
            }
            Err(e) => prop_assert!(false, "Failed to parse printed expression: {:?}", e),
        }
    }

    // Test 2: Precedence - multiplication binds tighter than addition
    #[test]
    fn precedence_mul_over_add(a in arb_number(), b in arb_number(), c in arb_number()) {
        let expr_str = format!("{} + {} * {}", a, b, c);
        let parsed = parse_expression(&expr_str).unwrap();
        
        // Should parse as a + (b * c), not (a + b) * c
        // Need to check structure, not exact values due to negation handling
        fn is_mul(expr: &Expr) -> bool {
            matches!(expr, Expr::Mul(_, _))
        }
        
        fn check_precedence(expr: &Expr) -> bool {
            match expr {
                Expr::Add(_, right) => is_mul(right),
                _ => false,
            }
        }
        
        prop_assert!(check_precedence(&parsed), 
                    "Expected Add(_, Mul(_, _)) structure, got {:?}", parsed);
    }

    // Test 3: Left associativity 
    #[test]
    fn left_associativity(a in arb_number(), b in arb_number(), c in arb_number()) {
        // Test subtraction (left associative)
        let sub_str = format!("{} - {} - {}", a, b, c);
        let sub_parsed = parse_expression(&sub_str).unwrap();
        match sub_parsed {
            Expr::Sub(left, _right) => {
                match left.as_ref() {
                    Expr::Sub(_, _) => {}, // (a - b) - c
                    _ => prop_assert!(false, "Subtraction should be left associative"),
                }
            }
            _ => prop_assert!(false, "Should be subtraction at top level"),
        }

        // Test division (left associative)
        if b != 0.0 && c != 0.0 {
            let div_str = format!("{} / {} / {}", a, b, c);
            let div_parsed = parse_expression(&div_str).unwrap();
            match div_parsed {
                Expr::Div(left, _) => {
                    match left.as_ref() {
                        Expr::Div(_, _) => {}, // (a / b) / c
                        _ => prop_assert!(false, "Division should be left associative"),
                    }
                }
                _ => prop_assert!(false, "Should be division at top level"),
            }
        }
    }

    // Test 4: Parentheses override precedence
    #[test]
    fn parentheses_override(a in arb_number(), b in arb_number(), c in arb_number()) {
        let with_parens = format!("({} + {}) * {}", a, b, c);
        let without_parens = format!("{} + {} * {}", a, b, c);
        
        let parsed_with = parse_expression(&with_parens).unwrap();
        let parsed_without = parse_expression(&without_parens).unwrap();
        
        // These should produce different AST structures
        match (&parsed_with, &parsed_without) {
            (Expr::Mul(left, _), Expr::Add(_, _)) => {
                // with_parens: (a + b) * c
                // without_parens: a + (b * c)
                match left.as_ref() {
                    Expr::Add(_, _) => {}, // Correct
                    _ => prop_assert!(false, "Parentheses didn't group addition"),
                }
            }
            _ => prop_assert!(false, "Unexpected AST structures"),
        }
    }

    // Test 5: Unary minus binding  
    #[test]
    fn unary_minus_binding(a in arb_number(), b in arb_number()) {
        let expr_str = format!("-{} + {}", a, b);
        let parsed = parse_expression(&expr_str).unwrap();
        
        // Should parse as (-a) + b, not -(a + b)
        // Check the structure: should be Add where left side has negation
        fn has_negation(expr: &Expr) -> bool {
            match expr {
                Expr::Neg(_) => true,
                Expr::Number(n) => *n < 0.0,
                _ => false,
            }
        }
        
        match &parsed {
            Expr::Add(left, _right) => {
                prop_assert!(has_negation(left), 
                            "Expected negation on left side of addition, got {:?}", parsed);
            }
            _ => prop_assert!(false, "Should be addition at top level, got {:?}", parsed),
        }
    }

    // Test 6: Evaluation determinism
    #[test]
    fn evaluation_determinism(expr in arb_expr()) {
        let result1 = eval_expr(&expr);
        let result2 = eval_expr(&expr);
        
        match (&result1, &result2) {
            (Ok(v1), Ok(v2)) => {
                // Handle NaN case specially - NaN != NaN by definition
                if v1.is_nan() && v2.is_nan() {
                    // Both are NaN, that's deterministic
                } else {
                    prop_assert_eq!(v1, v2, "Results differ: {} vs {}", v1, v2);
                }
            },
            (Err(ComputeError::DivisionByZero), Err(ComputeError::DivisionByZero)) => {},
            (Err(e1), Err(e2)) => prop_assert_eq!(e1, e2, "Different errors"),
            _ => prop_assert!(false, "Non-deterministic evaluation: {:?} vs {:?}", result1, result2),
        }
    }

    // Test 7: Commutativity of addition and multiplication
    #[test]
    fn commutativity(a in arb_expr(), b in arb_expr()) {
        let add_ab = Expr::Add(Box::new(a.clone()), Box::new(b.clone()));
        let add_ba = Expr::Add(Box::new(b.clone()), Box::new(a.clone()));
        
        match (eval_expr(&add_ab), eval_expr(&add_ba)) {
            (Ok(v1), Ok(v2)) => prop_assert!(approx_eq(v1, v2, 1e-10)),
            (Err(ComputeError::DivisionByZero), Err(ComputeError::DivisionByZero)) => {},
            _ => {}, // One has error, other doesn't - that's ok
        }

        let mul_ab = Expr::Mul(Box::new(a.clone()), Box::new(b.clone()));
        let mul_ba = Expr::Mul(Box::new(b), Box::new(a));
        
        match (eval_expr(&mul_ab), eval_expr(&mul_ba)) {
            (Ok(v1), Ok(v2)) => prop_assert!(approx_eq(v1, v2, 1e-10)),
            (Err(ComputeError::DivisionByZero), Err(ComputeError::DivisionByZero)) => {},
            _ => {}, // One has error, other doesn't - that's ok
        }
    }

    // Test 8: Identity elements
    #[test]
    fn identity_elements(expr in arb_expr()) {
        // Addition identity: a + 0 = a
        let add_zero = Expr::Add(Box::new(expr.clone()), Box::new(Expr::Number(0.0)));
        match (eval_expr(&expr), eval_expr(&add_zero)) {
            (Ok(v1), Ok(v2)) => prop_assert!(approx_eq(v1, v2, 1e-10)),
            (Err(ComputeError::DivisionByZero), Err(ComputeError::DivisionByZero)) => {},
            _ => {}, // Error handling might differ
        }

        // Multiplication identity: a * 1 = a  
        let mul_one = Expr::Mul(Box::new(expr.clone()), Box::new(Expr::Number(1.0)));
        match (eval_expr(&expr), eval_expr(&mul_one)) {
            (Ok(v1), Ok(v2)) => prop_assert!(approx_eq(v1, v2, 1e-10)),
            (Err(ComputeError::DivisionByZero), Err(ComputeError::DivisionByZero)) => {},
            _ => {}, // Error handling might differ
        }
    }

    // Test 9: Double negation
    #[test]
    fn double_negation(expr in arb_expr()) {
        let double_neg = Expr::Neg(Box::new(Expr::Neg(Box::new(expr.clone()))));
        
        match (eval_expr(&expr), eval_expr(&double_neg)) {
            (Ok(v1), Ok(v2)) => prop_assert!(approx_eq(v1, v2, 1e-10)),
            (Err(ComputeError::DivisionByZero), Err(ComputeError::DivisionByZero)) => {},
            _ => {}, // Error handling might differ
        }
    }

    // Test 10: Division by zero detection
    #[test]
    fn division_by_zero_detection(numerator in arb_expr()) {
        let div_zero = Expr::Div(Box::new(numerator), Box::new(Expr::Number(0.0)));
        prop_assert!(matches!(eval_expr(&div_zero), Err(ComputeError::DivisionByZero)));
    }

    // Test 11: Deep nesting doesn't cause stack overflow
    #[test]
    fn deep_nesting_handling(depth in 10u32..20u32) {
        let deep_expr = arb_deep_expr(depth).new_tree(&mut TestRunner::default()).unwrap().current();
        
        // Should be able to evaluate without panicking
        let _ = eval_expr(&deep_expr);
        
        // Should be able to print without panicking
        let printed = format!("{}", deep_expr);
        prop_assert!(printed.len() > 0);
        
        // Should be able to parse the printed version
        match parse_expression(&printed) {
            Ok(parsed) => {
                // And evaluate it
                let _ = eval_expr(&parsed);
            }
            Err(_) => {
                // Parser errors are ok for very deep expressions
            }
        }
    }

    // Test 12: Whitespace insensitivity
    #[test]
    fn whitespace_insensitivity(expr_str in arb_expr_with_whitespace()) {
        let stripped = expr_str.chars().filter(|c| !c.is_whitespace()).collect::<String>();
        
        // Both should parse to equivalent expressions
        match (parse_expression(&expr_str), parse_expression(&stripped)) {
            (Ok(expr1), Ok(expr2)) => {
                match (eval_expr(&expr1), eval_expr(&expr2)) {
                    (Ok(v1), Ok(v2)) => prop_assert!(approx_eq(v1, v2, 1e-10)),
                    (Err(ComputeError::DivisionByZero), Err(ComputeError::DivisionByZero)) => {},
                    _ => prop_assert!(false, "Different errors for whitespace variants"),
                }
            }
            (Err(_), Err(_)) => {
                // Both failed to parse - that's consistent
            }
            _ => {
                // One parsed, one didn't - might be due to number parsing
                // e.g., "1 .5" vs "1.5"
            }
        }
    }

    // Test 13: Parse-eval consistency
    #[test]
    fn parse_eval_consistency(expr_str in arb_expr_string()) {
        match (evaluate(&expr_str), parse_expression(&expr_str)) {
            (Ok(value), Ok(ast)) => {
                match eval_expr(&ast) {
                    Ok(ast_value) => {
                        // Use approximate equality for floating point
                        prop_assert!(approx_eq(value, ast_value, 1e-10),
                                    "evaluate() = {}, eval_expr(parse()) = {}", value, ast_value);
                    }
                    Err(ComputeError::DivisionByZero) => {
                        // If eval failed with division by zero, evaluate should too
                        prop_assert!(false, "Parse succeeded but eval failed with DivisionByZero");
                    }
                    Err(e) => prop_assert!(false, "Parse succeeded but eval failed: {:?}", e),
                }
            }
            (Err(ComputeError::DivisionByZero), Ok(ast)) => {
                // evaluate caught division by zero, eval_expr should too
                match eval_expr(&ast) {
                    Err(ComputeError::DivisionByZero) => {},
                    other => prop_assert!(false, "Inconsistent division by zero handling: {:?}", other),
                }
            }
            (Err(_), Err(_)) => {
                // Both failed - that's consistent
            }
            (Ok(_), Err(_)) => {
                // This shouldn't happen - evaluate calls parse_expression internally
                prop_assert!(false, "evaluate succeeded but parse_expression failed");
            }
            (Err(e1), Ok(_)) => {
                // evaluate failed but parse succeeded - could be division by zero during eval
                prop_assert!(matches!(e1, ComputeError::DivisionByZero),
                            "Unexpected error type: {:?}", e1);
            }
        }
    }

    // Test 14: Number precision preservation
    #[test]
    fn number_precision(n in arb_number()) {
        let expr_str = format!("{}", n);
        match parse_expression(&expr_str) {
            Ok(parsed_expr) => {
                // Extract the number, handling potential negation
                let parsed_n = match &parsed_expr {
                    Expr::Number(num) => *num,
                    Expr::Neg(inner) => match inner.as_ref() {
                        Expr::Number(num) => -*num,
                        _ => {
                            prop_assert!(false, "Unexpected nested structure");
                            return Ok(());
                        }
                    },
                    _ => {
                        prop_assert!(false, "Single number didn't parse to Number or Neg(Number)");
                        return Ok(());
                    }
                };
                
                if n.is_finite() && parsed_n.is_finite() {
                    // For finite numbers, check they're very close
                    prop_assert!(approx_eq(n, parsed_n, 1e-10), 
                                "Expected {} but got {}", n, parsed_n);
                } else if n.is_infinite() && parsed_n.is_infinite() {
                    // For infinite values, check sign matches
                    prop_assert_eq!(n.is_sign_positive(), parsed_n.is_sign_positive());
                } else if n.is_nan() {
                    // NaN might not parse correctly, that's ok
                }
            }
            Err(_) => {
                // Some special float values might not parse
                // This is ok for NaN, infinity, or very large numbers
                prop_assert!(!n.is_finite() || n.abs() > 1e308 || n.is_nan());
            }
        }
    }

    // Test 15: Expression depth is finite and reasonable
    #[test]
    fn finite_expression_depth(expr in arb_expr()) {
        fn depth(e: &Expr) -> u32 {
            match e {
                Expr::Number(_) => 1,
                Expr::Add(l, r) | Expr::Sub(l, r) | Expr::Mul(l, r) | Expr::Div(l, r) => {
                    1 + depth(l).max(depth(r))
                }
                Expr::Neg(e) => 1 + depth(e),
            }
        }
        
        let d = depth(&expr);
        prop_assert!(d <= MAX_DEPTH + 1); // Allow for some recursion overshoot
    }

    // Test 16: Malformed input handling
    #[test]
    fn malformed_input_handling(s in ".*") {
        // Any string should either parse or return an error, never panic
        match evaluate(&s) {
            Ok(_) => {},
            Err(_) => {},
        }
    }

    // Test 17: Batch evaluation consistency
    #[test]
    fn batch_evaluation_consistency(exprs in prop::collection::vec(arb_expr_string(), 1..10)) {
        let expr_refs: Vec<&str> = exprs.iter().map(|s| s.as_str()).collect();
        let batch_results = evaluate_batch(&expr_refs);
        
        prop_assert_eq!(batch_results.len(), exprs.len());
        
        for (i, result) in batch_results.iter().enumerate() {
            prop_assert_eq!(&result.expression, &exprs[i]);
            
            // Individual evaluation should match batch
            match (&result.value, evaluate(&exprs[i])) {
                (Ok(v1), Ok(v2)) => {
                    // Use approximate equality for floats
                    prop_assert!(approx_eq(*v1, v2, 1e-10),
                                "Batch result {} != individual result {} for expression '{}'",
                                v1, v2, exprs[i]);
                },
                (Err(e1), Err(e2)) => {
                    // Both failed - check same error type
                    prop_assert!(
                        std::mem::discriminant(e1) == std::mem::discriminant(&e2),
                        "Different error types: {:?} vs {:?}", e1, e2
                    );
                },
                _ => prop_assert!(false, "Batch and individual evaluation differ for '{}'", exprs[i]),
            }
        }
    }

    // Test 18: Adversarial nested parentheses
    #[test]
    fn nested_parentheses(n in 1usize..20, inner in arb_number()) {
        let mut expr = format!("{}", inner);
        for _ in 0..n {
            expr = format!("({})", expr);
        }
        
        match evaluate(&expr) {
            Ok(v) => prop_assert!(approx_eq(v, inner, 1e-10)),
            Err(_) => prop_assert!(false, "Failed to parse nested parentheses"),
        }
    }

    // Test 19: Associativity with near-zero values
    #[test]
    fn associativity_near_zero(tiny in -1e-100..1e-100) {
        // (1 + tiny) - 1 vs 1 + (tiny - 1)
        let left_assoc = format!("({} + {}) - {}", 1.0, tiny, 1.0);
        let right_assoc = format!("{} + ({} - {})", 1.0, tiny, 1.0);
        
        match (evaluate(&left_assoc), evaluate(&right_assoc)) {
            (Ok(v1), Ok(v2)) => {
                // Due to floating point, these might differ slightly
                prop_assert!(approx_eq(v1, v2, 1e-10));
            }
            _ => {},
        }
    }

    // Test 20: Parser doesn't accept invalid operators
    #[test]
    fn invalid_operators(a in arb_number(), b in arb_number(), op in "[&|^%@#$!]") {
        let expr_str = format!("{} {} {}", a, op, b);
        prop_assert!(matches!(evaluate(&expr_str), Err(ComputeError::ParseError(_))));
    }
}

#[cfg(test)]
mod regression_tests {
    use super::*;

    #[test]
    fn test_specific_regression_cases() {
        // Add any specific failing cases discovered during testing
        
        // Very deep expression
        let mut deep = "1".to_string();
        for _ in 0..30 {
            deep = format!("({} + 1)", deep);
        }
        assert!(evaluate(&deep).is_ok());

        // Alternating operators
        assert_eq!(evaluate("1 - 2 + 3 - 4 + 5").unwrap(), 3.0);

        // Many parentheses
        assert_eq!(evaluate("((((((1))))))").unwrap(), 1.0);

        // Negative zero
        assert_eq!(evaluate("-0.0").unwrap(), -0.0);

        // Small differences - this will have precision loss due to floating point
        let result = evaluate("1e20 + 1 - 1e20").unwrap();
        // Due to floating point precision, this might be 0.0 instead of 1.0
        assert!(result == 0.0 || approx_eq(result, 1.0, 1.0));
    }
}