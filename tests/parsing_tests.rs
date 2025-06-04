use compute_mcp::{evaluate, ComputeError};

#[test]
fn test_parse_errors_incomplete_expressions() {
    // Missing operands
    let incomplete = vec![
        "2 +",
        "+ 3",
        "2 * ",
        "/ 5",
        "2 + * 3",
        "* 2 + 3",
    ];
    
    for expr in incomplete {
        assert!(
            matches!(evaluate(expr), Err(ComputeError::ParseError(_))),
            "Expression '{}' should result in parse error",
            expr
        );
    }
}

#[test]
fn test_parse_errors_invalid_operators() {
    let invalid = vec![
        "2 & 3",
        "2 | 3",
        "2 ^ 3",
        "2 % 3",
        "2 ** 3",
        "2 // 3",
        "2 += 3",
    ];
    
    for expr in invalid {
        assert!(
            matches!(evaluate(expr), Err(ComputeError::ParseError(_))),
            "Expression '{}' should result in parse error",
            expr
        );
    }
}

#[test]
fn test_parse_errors_unmatched_parentheses() {
    let unmatched = vec![
        "(2 + 3",
        "2 + 3)",
        "((2 + 3)",
        "(2 + 3))",
        "2 + (3 + )",
        "()",
        "( )",
    ];
    
    for expr in unmatched {
        assert!(
            matches!(evaluate(expr), Err(ComputeError::ParseError(_))),
            "Expression '{}' should result in parse error",
            expr
        );
    }
}

#[test]
fn test_parse_errors_invalid_numbers() {
    let invalid_numbers = vec![
        "2.3.4",
        "2e10",  // Scientific notation not supported
        "1,000", // Comma separators not supported
        "0x10",  // Hex not supported
        "0b101", // Binary not supported
        ".5",    // Must have leading digit
        "5.",    // Must have trailing digit
    ];
    
    for expr in invalid_numbers {
        let result = evaluate(expr);
        assert!(
            result.is_err(),
            "Expression '{}' should result in error, got {:?}",
            expr,
            result
        );
    }
}

#[test]
fn test_parse_errors_invalid_identifiers() {
    let with_identifiers = vec![
        "abc",
        "x + 2",
        "2 + y",
        "sin(45)",
        "pi * 2",
        "e ^ 2",
        "sqrt(16)",
    ];
    
    for expr in with_identifiers {
        assert!(
            matches!(evaluate(expr), Err(ComputeError::ParseError(_))),
            "Expression '{}' should result in parse error",
            expr
        );
    }
}

#[test]
fn test_parse_errors_adjacent_numbers() {
    let adjacent = vec![
        "2 3",
        "2 3 4",
        "2(3)",  // Implicit multiplication not supported
        "(2)(3)",
    ];
    
    for expr in adjacent {
        assert!(
            matches!(evaluate(expr), Err(ComputeError::ParseError(_))),
            "Expression '{}' should result in parse error",
            expr
        );
    }
}

#[test]
fn test_parse_errors_special_characters() {
    let special = vec![
        "2 @ 3",
        "2 # 3",
        "2 $ 3",
        "2 ! 3",
        "[2 + 3]",
        "{2 + 3}",
        "2 + 3;",
        "2, 3",
    ];
    
    for expr in special {
        assert!(
            matches!(evaluate(expr), Err(ComputeError::ParseError(_))),
            "Expression '{}' should result in parse error",
            expr
        );
    }
}

#[test]
fn test_error_messages_contain_context() {
    // Test that parse errors contain helpful information
    let result = evaluate("2 +");
    assert!(result.is_err());
    if let Err(ComputeError::ParseError(e)) = result {
        let error_str = e.to_string();
        assert!(error_str.contains("expected") || error_str.contains("EOI"));
    }
    
    let result = evaluate("2 + + 3");
    assert!(result.is_err());
    if let Err(ComputeError::ParseError(e)) = result {
        let error_str = e.to_string();
        assert!(error_str.contains("expected"));
    }
}