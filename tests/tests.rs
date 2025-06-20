use compute_mcp::{evaluate, ComputeError, Expr};
use proptest::prelude::*;

// Basic sanity tests that are easy to read and understand
#[test]
fn basic_arithmetic() {
    assert_eq!(evaluate("2 + 2").unwrap(), 4.0);
    assert_eq!(evaluate("10 - 3").unwrap(), 7.0);
    assert_eq!(evaluate("4 * 5").unwrap(), 20.0);
    assert_eq!(evaluate("15 / 3").unwrap(), 5.0);
}

#[test]
fn operator_precedence() {
    assert_eq!(evaluate("2 + 3 * 4").unwrap(), 14.0); // Not 20
    assert_eq!(evaluate("10 - 2 * 3").unwrap(), 4.0); // Not 24
    assert_eq!(evaluate("(2 + 3) * 4").unwrap(), 20.0); // Parentheses override
}

#[test]
fn negative_numbers() {
    assert_eq!(evaluate("-5").unwrap(), -5.0);
    assert_eq!(evaluate("-5 + 3").unwrap(), -2.0);
    assert_eq!(evaluate("-(2 + 3)").unwrap(), -5.0);
}

#[test]
fn division_by_zero() {
    assert!(matches!(
        evaluate("10 / 0"),
        Err(ComputeError::DivisionByZero)
    ));
    assert!(matches!(
        evaluate("(5 - 5) / (3 - 3)"),
        Err(ComputeError::DivisionByZero)
    ));
}

#[test]
fn parse_errors() {
    assert!(matches!(evaluate(""), Err(ComputeError::EmptyExpression)));
    assert!(matches!(evaluate("2 +"), Err(ComputeError::ParseError(_))));
    assert!(matches!(
        evaluate("hello"),
        Err(ComputeError::ParseError(_))
    ));
}

// Direct evaluator for testing - this is our "obviously correct" reference
fn direct_eval(expr: &Expr) -> f64 {
    match expr {
        Expr::Number(n) => *n,
        Expr::Add(l, r) => direct_eval(l) + direct_eval(r),
        Expr::Sub(l, r) => direct_eval(l) - direct_eval(r),
        Expr::Mul(l, r) => direct_eval(l) * direct_eval(r),
        Expr::Div(l, r) => {
            let divisor = direct_eval(r);
            if divisor == 0.0 {
                f64::NAN
            } else {
                direct_eval(l) / divisor
            }
        }
        Expr::Neg(e) => -direct_eval(e),
    }
}

// Strategy for generating expression trees
fn arb_expr() -> impl Strategy<Value = Expr> {
    let leaf = (-100.0f64..100.0).prop_map(Expr::Number);

    leaf.prop_recursive(3, 20, 5, |inner| {
        prop_oneof![
            (inner.clone(), inner.clone()).prop_map(|(l, r)| Expr::Add(l.into(), r.into())),
            (inner.clone(), inner.clone()).prop_map(|(l, r)| Expr::Sub(l.into(), r.into())),
            (inner.clone(), inner.clone()).prop_map(|(l, r)| Expr::Mul(l.into(), r.into())),
            (inner.clone(), inner.clone()).prop_map(|(l, r)| Expr::Div(l.into(), r.into())),
            inner.prop_map(|e| Expr::Neg(e.into())),
        ]
    })
}

proptest! {
    #[test]
    fn round_trip_evaluation(expr in arb_expr()) {
        // Convert expression to string using Display
        let expr_str = expr.to_string();

        // Direct evaluation (our reference)
        let direct_result = direct_eval(&expr);

        // Skip if we hit division by zero
        if direct_result.is_nan() {
            return Ok(());
        }

        // Parse and evaluate through the full pipeline
        let parsed_result = evaluate(&expr_str);

        // Both should give the same result
        match parsed_result {
            Ok(value) => {
                assert!((value - direct_result).abs() < 0.0001,
                    "Mismatch for {}: parsed {} vs direct {}",
                    expr_str, value, direct_result);
            }
            Err(_) => {
                panic!("Failed to parse generated expression: {}", expr_str);
            }
        }
    }

    #[test]
    fn parser_never_panics(s in "\\PC*") {
        // Any string input should either parse or return an error, never panic
        let _ = evaluate(&s);
    }
}
