#![allow(clippy::approx_constant)]

use compute_mcp::{Expression, evaluate_batch, ComputeError};

#[test]
fn test_batch_evaluation_basic() {
    let test_cases: Vec<(&str, Result<f64, ComputeError>)> = vec![
        ("2 + 3", Ok(5.0)),
        ("10 * 5", Ok(50.0)),
        ("100 / 20", Ok(5.0)),
        ("15 - 7", Ok(8.0)),
    ];
    
    let expressions: Vec<Expression> = test_cases.iter()
        .map(|(expr, _)| Expression::from(*expr))
        .collect();
    
    let results = evaluate_batch(&expressions);
    assert_eq!(results.len(), test_cases.len());
    
    for ((expr_str, expected), result) in test_cases.iter().zip(results.iter()) {
        // Verify expression is preserved
        assert_eq!(result.expression.as_str(), *expr_str);
        
        // Verify result matches expected
        match (&result.value, expected) {
            (Ok(actual), Ok(expected)) => {
                assert_eq!(actual, expected, "Expression '{}' failed", expr_str);
            }
            (Err(_), Err(_)) => {
                // Both are errors, that's what we expected
            }
            _ => panic!("Expression '{}' result mismatch: got {:?}, expected {:?}", 
                       expr_str, result.value, expected),
        }
    }
}

#[test]
fn test_batch_evaluation_mixed_results() {
    // Test cases with expected results (success or specific error types)
    let test_cases = [
        ("10 + 5", Ok(15.0)),
        ("20 / 0", Err(ComputeError::DivisionByZero)),
        ("3.14 * 2", Ok(6.28)),
        ("(5 + 3) * 2", Ok(16.0)),
    ];
    
    let expressions: Vec<Expression> = test_cases.iter()
        .map(|(expr, _)| Expression::from(*expr))
        .collect();
    
    let results = evaluate_batch(&expressions);
    assert_eq!(results.len(), test_cases.len());
    
    for ((expr_str, expected), result) in test_cases.iter().zip(results.iter()) {
        assert_eq!(result.expression.as_str(), *expr_str);
        
        match (&result.value, expected) {
            (Ok(actual), Ok(expected_val)) => {
                assert!(
                    (actual - expected_val).abs() < 0.0001,
                    "Expression '{}' expected {} but got {}",
                    expr_str, expected_val, actual
                );
            }
            (Err(ComputeError::DivisionByZero), Err(ComputeError::DivisionByZero)) => {
                // Expected division by zero error
            }
            (Err(ComputeError::ParseError(_)), Err(ComputeError::ParseError(_))) => {
                // Expected parse error
            }
            _ => panic!("Expression '{}' result mismatch: got {:?}, expected {:?}", 
                       expr_str, result.value, expected),
        }
    }
    
    // Test parse error separately since we can't construct ParseError directly
    let parse_error_expr = Expression::from("invalid");
    let parse_error_results = evaluate_batch(&[parse_error_expr]);
    assert!(matches!(parse_error_results[0].value, Err(ComputeError::ParseError(_))));
}

#[test]
fn test_batch_evaluation_empty() {
    let expressions: Vec<Expression> = vec![];
    let results = evaluate_batch(&expressions);
    assert_eq!(results.len(), 0);
}

#[test]
fn test_batch_evaluation_all_errors() {
    let test_cases: Vec<(&str, ComputeError)> = vec![
        ("10 / 0", ComputeError::DivisionByZero),
        ("5 / (3 - 3)", ComputeError::DivisionByZero),
    ];
    
    let expressions: Vec<Expression> = test_cases.iter()
        .map(|(expr, _)| Expression::from(*expr))
        .collect();
    
    let results = evaluate_batch(&expressions);
    assert_eq!(results.len(), test_cases.len());
    
    for ((expr_str, expected_error), result) in test_cases.iter().zip(results.iter()) {
        assert!(result.value.is_err(), "Expected error for '{}'", expr_str);
        
        match (&result.value, expected_error) {
            (Err(ComputeError::DivisionByZero), ComputeError::DivisionByZero) => {},
            _ => panic!("Expression '{}' expected {:?} but got {:?}", 
                       expr_str, expected_error, result.value),
        }
    }
    
    // Test parse errors separately (can't construct them directly)
    let parse_error_cases = vec![
        Expression::from("invalid expression"),
        Expression::from("2 + + 3"),
    ];
    
    let parse_results = evaluate_batch(&parse_error_cases);
    for result in parse_results {
        assert!(matches!(result.value, Err(ComputeError::ParseError(_))));
    }
}

#[test]
fn test_batch_evaluation_large_batch() {
    // Create a large batch of expressions
    let mut expressions = Vec::new();
    for i in 1..=100 {
        expressions.push(Expression::from(format!("{} + {}", i, i).as_str()));
    }
    
    let results = evaluate_batch(&expressions);
    
    assert_eq!(results.len(), 100);
    for (i, result) in results.iter().enumerate() {
        let expected = ((i + 1) * 2) as f64;
        assert_eq!(
            result.value.as_ref().unwrap(),
            &expected,
            "Failed at index {}",
            i
        );
    }
}

#[test]
fn test_batch_preserves_expression_order() {
    let test_cases = [
        ("5 * 5", 25.0),
        ("3 + 2", 5.0),
        ("10 - 1", 9.0),
        ("20 / 4", 5.0),
    ];
    
    let expressions: Vec<Expression> = test_cases.iter()
        .map(|(expr, _)| Expression::from(*expr))
        .collect();
    
    let results = evaluate_batch(&expressions);
    
    // Verify order is preserved
    for ((expr_str, expected_val), result) in test_cases.iter().zip(results.iter()) {
        assert_eq!(result.expression.as_str(), *expr_str);
        assert_eq!(result.value.as_ref().unwrap(), expected_val);
    }
}

#[test]
fn test_batch_evaluation_complex_expressions() {
    let test_cases = [
        ("((2 + 3) * 4 - 5) / (6 - 1)", 3.0),
        ("2 * (3 + (4 * 5))", 46.0),
        ("-(10 - 15) * 2", 10.0),
        ("3.14159 * 2 * 5", 31.4159),
    ];
    
    let expressions: Vec<Expression> = test_cases.iter()
        .map(|(expr, _)| Expression::from(*expr))
        .collect();
    
    let results = evaluate_batch(&expressions);
    assert_eq!(results.len(), test_cases.len());
    
    for ((expr_str, expected_val), result) in test_cases.iter().zip(results.iter()) {
        let result_val = result.value.as_ref().unwrap();
        assert!(
            (result_val - expected_val).abs() < 0.0001,
            "Expression '{}' expected {} but got {}",
            expr_str, expected_val, result_val
        );
    }
}