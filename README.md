# compute-mcp

A minimal arithmetic MCP (Model Context Protocol) server demonstrating how to build MCP tools. This example is designed to be concise and blog-friendly while showing the core patterns.

## Features

- Basic arithmetic operations: `+`, `-`, `*`, `/`
- Correct operator precedence
- Parentheses for grouping
- Batch evaluation of multiple expressions

## Grammar

The arithmetic grammar (in `src/compute.pest`) supports:
- Numbers: `42`, `-3.14`, `0.5`
- Addition/Subtraction: `2 + 3`, `10 - 4`
- Multiplication/Division: `3 * 4`, `15 / 3`
- Parentheses: `(2 + 3) * 4`
- Proper precedence: `2 + 3 * 4` evaluates as `2 + (3 * 4) = 14`

## MCP Tool

### evaluate_batch
Evaluate multiple expressions at once:
```json
{
  "name": "evaluate_batch",
  "arguments": {
    "expressions": ["2 + 2", "10 / 5", "(3 + 4) * 2"]
  }
}
```

Response:
```json
{
  "success": true,
  "results": [
    {"expression": "2 + 2", "result": 4.0, "error": null, "success": true},
    {"expression": "10 / 5", "result": 2.0, "error": null, "success": true},
    {"expression": "(3 + 4) * 2", "result": 14.0, "error": null, "success": true}
  ]
}
```

## Building

```bash
cargo build --release
```

The MCP server binary will be at `target/release/stdio_direct`.

## Testing

Run unit tests and property tests:
```bash
cargo test
```

## Running

For direct stdio testing:
```bash
cargo run --bin stdio_direct
```

For use with Claude Desktop, add to your configuration:
```json
{
  "mcpServers": {
    "compute": {
      "command": "/path/to/compute-mcp/target/release/stdio_direct"
    }
  }
}
```

## Architecture

1. **Parser** (`lib.rs`): Uses Pest to parse arithmetic expressions into an AST
2. **Evaluator** (`lib.rs`): Recursively evaluates the AST to compute results
3. **MCP Server** (`src/bin/stdio_direct.rs`): Implements MCP protocol with JSON-RPC
4. **Property Tests** (`tests/property_tests.rs`): Verify mathematical properties hold

The implementation follows the direct JSON-RPC handling pattern used by kv-memory-mcp and popup-mcp, providing a minimal example of MCP tool development.
