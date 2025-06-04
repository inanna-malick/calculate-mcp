use compute_mcp::{parse_expression, Expression, Expr, evaluate, eval_expr};

#[test]
fn test_ast_construction() {
    // Number
    let expr = parse_expression(&Expression::from("42")).unwrap();
    match expr {
        Expr::Number(n) => assert_eq!(n, 42.0),
        _ => panic!("Expected Number, got {:?}", expr),
    }
    
    // Addition
    let expr = parse_expression(&Expression::from("2 + 3")).unwrap();
    match expr {
        Expr::Add(left, right) => {
            match (left.as_ref(), right.as_ref()) {
                (Expr::Number(2.0), Expr::Number(3.0)) => {},
                _ => panic!("Expected Add(2, 3)"),
            }
        }
        _ => panic!("Expected Add, got {:?}", expr),
    }
    
    // Nested expression
    let expr = parse_expression(&Expression::from("2 + 3 * 4")).unwrap();
    match expr {
        Expr::Add(left, right) => {
            match (left.as_ref(), right.as_ref()) {
                (Expr::Number(2.0), Expr::Mul(ml, mr)) => {
                    match (ml.as_ref(), mr.as_ref()) {
                        (Expr::Number(3.0), Expr::Number(4.0)) => {},
                        _ => panic!("Expected Mul(3, 4)"),
                    }
                }
                _ => panic!("Expected Add(2, Mul(3, 4))"),
            }
        }
        _ => panic!("Expected Add, got {:?}", expr),
    }
}

#[test]
fn test_ast_negation() {
    // Simple negation
    let expr = parse_expression(&Expression::from("-5")).unwrap();
    match expr {
        Expr::Neg(inner) => {
            match inner.as_ref() {
                Expr::Number(5.0) => {},
                _ => panic!("Expected Neg(5)"),
            }
        }
        _ => panic!("Expected Neg, got {:?}", expr),
    }
    
    // Negation of expression
    let expr = parse_expression(&Expression::from("-(2 + 3)")).unwrap();
    match expr {
        Expr::Neg(inner) => {
            match inner.as_ref() {
                Expr::Add(left, right) => {
                    match (left.as_ref(), right.as_ref()) {
                        (Expr::Number(2.0), Expr::Number(3.0)) => {},
                        _ => panic!("Expected Add(2, 3) inside Neg"),
                    }
                }
                _ => panic!("Expected Add inside Neg"),
            }
        }
        _ => panic!("Expected Neg, got {:?}", expr),
    }
}

#[test]
fn test_ast_all_operators() {
    // Test each operator type
    let test_cases = vec![
        ("5 + 3", "Add"),
        ("5 - 3", "Sub"),
        ("5 * 3", "Mul"),
        ("5 / 3", "Div"),
    ];
    
    for (expr_str, expected_op) in test_cases {
        let expr = parse_expression(&Expression::from(expr_str)).unwrap();
        let op_name = match expr {
            Expr::Add(_, _) => "Add",
            Expr::Sub(_, _) => "Sub",
            Expr::Mul(_, _) => "Mul",
            Expr::Div(_, _) => "Div",
            _ => "Other",
        };
        assert_eq!(op_name, expected_op, "Failed for expression: {}", expr_str);
    }
}

#[test]
fn test_ast_deeply_nested() {
    // Test deeply nested expression: ((2 + 3) * (4 - 1)) / 5
    let expr = parse_expression(&Expression::from("((2 + 3) * (4 - 1)) / 5")).unwrap();
    
    // Should be: Div(Mul(Add(2, 3), Sub(4, 1)), 5)
    match expr {
        Expr::Div(left, right) => {
            // Right should be 5
            match right.as_ref() {
                Expr::Number(5.0) => {},
                _ => panic!("Expected 5 on right of division"),
            }
            
            // Left should be Mul
            match left.as_ref() {
                Expr::Mul(ml, mr) => {
                    // ml should be Add(2, 3)
                    match ml.as_ref() {
                        Expr::Add(al, ar) => {
                            match (al.as_ref(), ar.as_ref()) {
                                (Expr::Number(2.0), Expr::Number(3.0)) => {},
                                _ => panic!("Expected Add(2, 3)"),
                            }
                        }
                        _ => panic!("Expected Add on left of Mul"),
                    }
                    
                    // mr should be Sub(4, 1)
                    match mr.as_ref() {
                        Expr::Sub(sl, sr) => {
                            match (sl.as_ref(), sr.as_ref()) {
                                (Expr::Number(4.0), Expr::Number(1.0)) => {},
                                _ => panic!("Expected Sub(4, 1)"),
                            }
                        }
                        _ => panic!("Expected Sub on right of Mul"),
                    }
                }
                _ => panic!("Expected Mul on left of Div"),
            }
        }
        _ => panic!("Expected Div at top level"),
    }
}

#[test]
fn test_ast_evaluation_consistency() {
    // Test that AST evaluation matches direct evaluation
    let test_cases = vec![
        "42",
        "-5",
        "2 + 3",
        "10 - 4",
        "3 * 4",
        "15 / 3",
        "2 + 3 * 4",
        "(2 + 3) * 4",
        "-(10 - 15)",
        "((2 + 3) * 4 - 5) / (6 - 1)",
    ];
    
    for expr_str in test_cases {
        let direct_result = evaluate(expr_str).unwrap();
        
        let expr = Expression::from(expr_str);
        let ast = parse_expression(&expr).unwrap();
        let ast_result = eval_expr(&ast).unwrap();
        
        assert_eq!(
            direct_result, ast_result,
            "Results differ for expression: {}",
            expr_str
        );
    }
}

#[test] 
fn test_ast_invalid_structure_errors() {
    // This test would require constructing invalid ASTs programmatically,
    // but since our parser ensures valid ASTs, we test the error path
    // through parse errors instead
    
    let invalid_exprs = vec![
        "",           // Empty
        "2 +",        // Incomplete
        "+ 3",        // Missing left operand
        "2 + + 3",    // Double operator
        "(2 + 3",     // Unclosed paren
    ];
    
    for expr_str in invalid_exprs {
        let expr = Expression::from(expr_str);
        let result = parse_expression(&expr);
        assert!(
            result.is_err(),
            "Expected parse error for: {}",
            expr_str
        );
    }
}