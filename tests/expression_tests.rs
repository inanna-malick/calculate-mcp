use compute_mcp::{Expression, parse_expression};

#[test]
fn test_expression_type_creation() {
    // Test empty expressions
    assert!(Expression::new("").is_none());
    assert!(Expression::new("  ").is_none());
    assert!(Expression::new("\t\n  ").is_none());
    
    // Test valid expressions
    assert!(Expression::new("2+3").is_some());
    assert!(Expression::new(" 2 + 3 ").is_some());
    
    // Test string preservation
    let expr = Expression::from("2 + 3");
    assert_eq!(expr.as_str(), "2 + 3");
    assert_eq!(expr.to_string(), "2 + 3");
    
    // Test Display trait
    let display = format!("{}", expr);
    assert_eq!(display, "2 + 3");
}

#[test]
fn test_expression_pretty_printer() {
    // Simple numbers
    let expr = parse_expression(&Expression::from("42")).unwrap();
    assert_eq!(expr.to_string(), "42");
    
    let expr = parse_expression(&Expression::from("3.14")).unwrap();
    assert_eq!(expr.to_string(), "3.14");
    
    let expr = parse_expression(&Expression::from("-5")).unwrap();
    assert_eq!(expr.to_string(), "-5");
    
    // Binary operations with parentheses
    let expr = parse_expression(&Expression::from("2 + 3")).unwrap();
    assert_eq!(expr.to_string(), "(2 + 3)");
    
    let expr = parse_expression(&Expression::from("10 - 4")).unwrap();
    assert_eq!(expr.to_string(), "(10 - 4)");
    
    let expr = parse_expression(&Expression::from("3 * 4")).unwrap();
    assert_eq!(expr.to_string(), "(3 * 4)");
    
    let expr = parse_expression(&Expression::from("15 / 3")).unwrap();
    assert_eq!(expr.to_string(), "(15 / 3)");
    
    // Nested operations
    let expr = parse_expression(&Expression::from("2 + 3 * 4")).unwrap();
    assert_eq!(expr.to_string(), "(2 + (3 * 4))");
    
    let expr = parse_expression(&Expression::from("(2 + 3) * 4")).unwrap();
    assert_eq!(expr.to_string(), "((2 + 3) * 4)");
    
    let expr = parse_expression(&Expression::from("2 * 3 + 4 * 5")).unwrap();
    assert_eq!(expr.to_string(), "((2 * 3) + (4 * 5))");
    
    // Complex nesting
    let expr = parse_expression(&Expression::from("((2 + 3) * 4 - 5) / 6")).unwrap();
    assert_eq!(expr.to_string(), "((((2 + 3) * 4) - 5) / 6)");
    
    // Negation
    let expr = parse_expression(&Expression::from("-(2 + 3)")).unwrap();
    assert_eq!(expr.to_string(), "-(2 + 3)");
    
    let expr = parse_expression(&Expression::from("-(-5)")).unwrap();
    assert_eq!(expr.to_string(), "--5");
}

#[test]
fn test_pretty_printer_number_formatting() {
    // Test integer formatting (no trailing .0)
    let expr = parse_expression(&Expression::from("42.0")).unwrap();
    assert_eq!(expr.to_string(), "42");
    
    // Test decimal preservation
    let expr = parse_expression(&Expression::from("3.14159")).unwrap();
    assert_eq!(expr.to_string(), "3.14159");
    
    // Test very small numbers
    let expr = parse_expression(&Expression::from("0.00001")).unwrap();
    assert_eq!(expr.to_string(), "0.00001");
    
    // Test negative numbers
    let expr = parse_expression(&Expression::from("-42.0")).unwrap();
    assert_eq!(expr.to_string(), "-42");
    
    let expr = parse_expression(&Expression::from("-3.14")).unwrap();
    assert_eq!(expr.to_string(), "-3.14");
}