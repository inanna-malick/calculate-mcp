//! 🔮 Crystalline MCP server

use anyhow::Result;
use compute_mcp::{Expression, evaluate_batch};
use mcpr::schema::json_rpc::{JSONRPCMessage, JSONRPCResponse};
use serde::Serialize;
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};

const GRAMMAR: &str = include_str!("../compute.pest");

// 💎 Response types
#[derive(Serialize)]
struct BatchResult {
    expression: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
    success: bool,
}


// 🎯 Response builders
fn success<T: Serialize>(data: T) -> Value {
    match serde_json::to_value(data) {
        Ok(value) => {
            if let Some(obj) = value.as_object() {
                let mut result = serde_json::Map::new();
                result.insert("success".to_string(), json!(true));
                result.extend(obj.clone());
                Value::Object(result)
            } else {
                json!({ "success": true, "data": value })
            }
        }
        Err(e) => json!({ "error": format!("Serialization failed: {}", e) })
    }
}

fn error(msg: impl std::fmt::Display) -> Value {
    json!({ "error": msg.to_string() })
}

fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .target(env_logger::Target::Stderr)
        .init();

    log::info!("🔮 Compute MCP starting...");

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    for line in stdin.lock().lines().flatten() {
        if line.trim().is_empty() { continue; }

        match serde_json::from_str::<JSONRPCMessage>(&line) {
            Ok(JSONRPCMessage::Request(req)) => {

                let response = match req.method.as_str() {
                    "initialize" => JSONRPCResponse::new(req.id, json!({
                        "protocolVersion": "2024-11-05",
                        "capabilities": {
                            "arithmetic": {
                                "operations": ["+", "-", "*", "/"],
                                "features": ["precedence", "parentheses", "decimals", "negatives", "div-by-zero"],
                                "grammar": GRAMMAR
                            }
                        },
                        "serverInfo": {
                            "name": "compute-mcp",
                            "version": "0.1.0",
                            "vibes": "🔮🎀💎"
                        }
                    })),
                    "tools/list" => JSONRPCResponse::new(req.id, json!({
                        "tools": [{
                            "name": "evaluate_batch",
                            "description": "Batch arithmetic evaluation",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "expressions": {
                                        "type": "array",
                                        "items": { "type": "string" }
                                    }
                                },
                                "required": ["expressions"]
                            }
                        }]
                    })),
                    "resources/list" => JSONRPCResponse::new(req.id, json!({"resources": []})),
                    "prompts/list" => JSONRPCResponse::new(req.id, json!({"prompts": []})),
                    "tools/call" => {
                        let params = req.params.unwrap_or(Value::Null);
                        let result = match params.get("name").and_then(|n| n.as_str()) {
                            Some("evaluate_batch") => {
                                params.get("arguments")
                                    .and_then(|args| args.get("expressions"))
                                    .and_then(|e| e.as_array())
                                    .map(|exprs| {
                                        let results: Vec<_> = exprs.iter()
                                            .filter_map(|v| v.as_str())
                                            .map(Expression::from)
                                            .flat_map(|expr| {
                                                evaluate_batch(&[expr]).into_iter().map(|r| {
                                                    BatchResult {
                                                        expression: r.expression.to_string(),
                                                        result: r.value.as_ref().ok().copied(),
                                                        error: r.value.as_ref().err().map(|e| e.to_string()),
                                                        success: r.value.is_ok(),
                                                    }
                                                })
                                            })
                                            .collect();
                                        success(json!({ "results": results }))
                                    })
                                    .unwrap_or_else(|| error("expressions must be array"))
                            }
                            _ => error("Unknown tool"),
                        };

                        JSONRPCResponse::new(req.id, json!({
                            "content": [{
                                "type": "text",
                                "text": serde_json::to_string(&result)?
                            }]
                        }))
                    }
                    _ => {
                        log::debug!("Unknown method: {}", req.method);
                        continue;
                    }
                };

                // 🎯 Send response
                writeln!(stdout, "{}", serde_json::to_string(&JSONRPCMessage::Response(response))?)?;
                stdout.flush()?;
            }
            Ok(_) => {}
            Err(e) => log::error!("Parse error: {}", e),
        }
    }

    log::info!("🎀 Server complete");
    Ok(())
}
