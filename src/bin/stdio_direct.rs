//! Direct stdio MCP server for arithmetic evaluation

use anyhow::Result;
use compute_mcp::{evaluate, Expression, evaluate_batch};
use mcpr::schema::json_rpc::{JSONRPCMessage, JSONRPCResponse};
use serde::Serialize;
use serde_json::Value;
use std::io::{self, BufRead, Write};
use std::sync::{Arc, Mutex};

// Include the grammar documentation
const COMPUTE_GRAMMAR: &str = include_str!("../compute.pest");

// Response structures
#[derive(Serialize)]
struct SuccessResponse<T: Serialize> {
    success: bool,
    #[serde(flatten)]
    data: T,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

#[derive(Serialize)]
struct EvaluateResponse {
    expression: String,
    result: f64,
}

#[derive(Serialize)]
struct BatchResult {
    expression: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
    success: bool,
}

#[derive(Serialize)]
struct BatchResponse {
    results: Vec<BatchResult>,
}

#[derive(Serialize)]
struct HistoryEntry {
    expression: String,
    result: f64,
}

#[derive(Serialize)]
struct HistoryResponse {
    history: Vec<HistoryEntry>,
    count: usize,
}

// Server state
struct ComputeState {
    history: Vec<(String, f64)>,
}

impl ComputeState {
    fn new() -> Self {
        Self {
            history: Vec::new(),
        }
    }

    fn add_to_history(&mut self, expression: String, result: f64) {
        self.history.push((expression, result));
        // Keep last 100 calculations
        if self.history.len() > 100 {
            self.history.remove(0);
        }
    }

    fn get_history(&self) -> Vec<HistoryEntry> {
        self.history
            .iter()
            .map(|(expr, result)| HistoryEntry {
                expression: expr.clone(),
                result: *result,
            })
            .collect()
    }
}

// Helper functions
fn success<T: Serialize>(data: T) -> Value {
    serde_json::to_value(SuccessResponse {
        success: true,
        data,
    })
    .unwrap()
}

fn error(msg: impl std::fmt::Display) -> Value {
    serde_json::to_value(ErrorResponse {
        error: msg.to_string(),
    })
    .unwrap()
}

fn main() -> Result<()> {
    // Set up logging to stderr
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .target(env_logger::Target::Stderr)
        .init();

    log::info!("Compute MCP server starting...");

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    // Initialize state
    let state = Arc::new(Mutex::new(ComputeState::new()));

    // Read messages line by line
    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        log::debug!("Received: {}", line);

        // Parse the JSON-RPC message
        match serde_json::from_str::<JSONRPCMessage>(&line) {
            Ok(JSONRPCMessage::Request(req)) => {
                log::info!("Request: {} (id: {:?})", req.method, req.id);

                let response = match req.method.as_str() {
                    "initialize" => {
                        log::debug!("Handling initialization");
                        JSONRPCResponse::new(
                            req.id,
                            serde_json::json!({
                                "protocolVersion": "2024-11-05",
                                "capabilities": {
                                    "arithmetic": {
                                        "description": "Basic arithmetic expression evaluation",
                                        "operations": ["+", "-", "*", "/"],
                                        "features": [
                                            "Correct operator precedence",
                                            "Parentheses for grouping",
                                            "Decimal number support",
                                            "Negative number support",
                                            "Division by zero detection"
                                        ],
                                        "grammar": COMPUTE_GRAMMAR
                                    }
                                },
                                "serverInfo": {
                                    "name": "compute-mcp",
                                    "version": "0.1.0",
                                    "description": "Minimal arithmetic MCP server for blog example"
                                }
                            }),
                        )
                    }
                    "tools/list" => {
                        log::debug!("Handling tools/list");
                        JSONRPCResponse::new(
                            req.id,
                            serde_json::json!({
                                "tools": [
                                    {
                                        "name": "evaluate",
                                        "description": "Evaluate an arithmetic expression",
                                        "inputSchema": {
                                            "type": "object",
                                            "properties": {
                                                "expression": {
                                                    "type": "string",
                                                    "description": "Arithmetic expression to evaluate (e.g., '2 + 3 * 4')"
                                                }
                                            },
                                            "required": ["expression"]
                                        }
                                    },
                                    {
                                        "name": "evaluate_batch",
                                        "description": "Evaluate multiple arithmetic expressions",
                                        "inputSchema": {
                                            "type": "object",
                                            "properties": {
                                                "expressions": {
                                                    "type": "array",
                                                    "items": {
                                                        "type": "string"
                                                    },
                                                    "description": "Array of arithmetic expressions to evaluate"
                                                }
                                            },
                                            "required": ["expressions"]
                                        }
                                    },
                                    {
                                        "name": "history",
                                        "description": "Get calculation history (last 100 calculations)",
                                        "inputSchema": {
                                            "type": "object",
                                            "properties": {}
                                        }
                                    }
                                ]
                            }),
                        )
                    }
                    "resources/list" => {
                        log::debug!("Handling resources/list");
                        JSONRPCResponse::new(req.id, serde_json::json!({"resources": []}))
                    }
                    "prompts/list" => {
                        log::debug!("Handling prompts/list");
                        JSONRPCResponse::new(req.id, serde_json::json!({"prompts": []}))
                    }
                    "tools/call" => {
                        log::debug!("Tool call params: {:?}", req.params);

                        let params = req.params.unwrap_or(Value::Null);
                        let tool_name = params.get("name").and_then(|n| n.as_str()).unwrap_or("");
                        let tool_args = params.get("arguments").cloned().unwrap_or(Value::Null);

                        let result = match tool_name {
                            "evaluate" => {
                                let expr_str = tool_args
                                    .get("expression")
                                    .and_then(|e| e.as_str())
                                    .unwrap_or("");

                                match evaluate(expr_str) {
                                    Ok(result) => {
                                        // Add to history
                                        if let Ok(mut state) = state.lock() {
                                            state.add_to_history(expr_str.to_string(), result);
                                        }

                                        success(EvaluateResponse {
                                            expression: expr_str.to_string(),
                                            result,
                                        })
                                    }
                                    Err(e) => error(e),
                                }
                            }
                            "evaluate_batch" => {
                                let expr_array = tool_args
                                    .get("expressions")
                                    .and_then(|e| e.as_array());

                                match expr_array {
                                    Some(exprs) => {
                                        // Convert JSON values to Expression objects
                                        let expressions: Vec<Expression> = exprs
                                            .iter()
                                            .filter_map(|v| v.as_str())
                                            .map(Expression::from)
                                            .collect();

                                        // Evaluate all expressions
                                        let batch_results = evaluate_batch(&expressions);
                                        
                                        // Convert results to response format
                                        let mut results = Vec::new();
                                        for eval_result in batch_results {
                                            match eval_result.value {
                                                Ok(result) => {
                                                    // Add to history
                                                    if let Ok(mut state) = state.lock() {
                                                        state.add_to_history(
                                                            eval_result.expression.to_string(),
                                                            result,
                                                        );
                                                    }

                                                    results.push(BatchResult {
                                                        expression: eval_result.expression.to_string(),
                                                        result: Some(result),
                                                        error: None,
                                                        success: true,
                                                    });
                                                }
                                                Err(e) => {
                                                    results.push(BatchResult {
                                                        expression: eval_result.expression.to_string(),
                                                        result: None,
                                                        error: Some(e.to_string()),
                                                        success: false,
                                                    });
                                                }
                                            }
                                        }

                                        success(BatchResponse { results })
                                    }
                                    None => error("expressions must be an array of strings"),
                                }
                            }
                            "history" => {
                                if let Ok(state) = state.lock() {
                                    let history = state.get_history();
                                    let count = history.len();
                                    success(HistoryResponse { history, count })
                                } else {
                                    error("Failed to access history")
                                }
                            }
                            _ => error(format!("Unknown tool: {}", tool_name)),
                        };

                        JSONRPCResponse::new(
                            req.id,
                            serde_json::json!({
                                "content": [{
                                    "type": "text",
                                    "text": serde_json::to_string(&result)?
                                }]
                            }),
                        )
                    }
                    _ => {
                        log::debug!("Unknown method: {}", req.method);
                        continue;
                    }
                };

                // Send response
                let response_str = serde_json::to_string(&JSONRPCMessage::Response(response))?;
                stdout.write_all(response_str.as_bytes())?;
                stdout.write_all(b"\n")?;
                stdout.flush()?;
                log::debug!("Sent response for {}", req.method);
            }
            Ok(msg) => {
                log::debug!("Other message type: {:?}", msg);
            }
            Err(e) => {
                log::error!("Parse error: {}", e);
            }
        }
    }

    log::info!("Server exiting");
    Ok(())
}
