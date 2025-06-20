# compute-mcp

A minimal arithmetic expression evaluator implemented as an MCP (Model Context Protocol) server. This project demonstrates a clean architecture for building language tools using Pest parser generator.

## Architecture

The project follows a clear data flow pattern:

```
Input String → Pest Parser → AST (Expr enum) → Evaluator → Result<f64>
     │              │              │                │            │
"2 + 3 * 4"    compute.pest    Expr::Add       eval_expr()    Ok(14.0)
                               /        \
                         Expr::Num(2)  Expr::Mul
                                       /        \
                                 Expr::Num(3)  Expr::Num(4)
```

### Components

1. **Grammar** (`src/compute.pest`) - Defines the syntax using Pest
2. **Parser** (`src/lib.rs`) - Converts strings to AST using the grammar
3. **AST** (`Expr` enum) - Represents expressions as a tree structure
4. **Evaluator** (`eval_expr`) - Recursively evaluates the AST
5. **MCP Server** (`src/bin/stdio_direct.rs`) - Exposes functionality via JSON-RPC

## Features

- Basic arithmetic operations: `+`, `-`, `*`, `/`
- Correct operator precedence (multiplication before addition)
- Parentheses for grouping: `(2 + 3) * 4`
- Decimal numbers: `3.14159`
- Negative numbers: `-42` or `-(5 + 3)`
- Error handling for division by zero

## Installation

```bash
cargo build --release
```

## Usage

### Command Line Testing

```bash
# Initialize the MCP server
echo '{"jsonrpc":"2.0","method":"initialize","params":{},"id":1}' | cargo run --bin stdio_direct

# Evaluate expressions
echo '{"jsonrpc":"2.0","method":"tools/call","params":{"name":"evaluate_batch","arguments":{"expressions":["2+2","3*4","10/2"]}},"id":2}' | cargo run --bin stdio_direct
```

### Claude Desktop Integration

Add to your Claude Desktop configuration:

```json
{
  "mcpServers": {
    "compute": {
      "command": "/path/to/compute-mcp/target/release/stdio_direct"
    }
  }
}
```

## Code Example

Here's how the evaluation pipeline works:

```rust
// 1. Parse string to AST
let expr = parse_expression(&Expression::from("2 + 3 * 4"))?;
// Result: Expr::Add(
//   Box::new(Expr::Number(2.0)),
//   Box::new(Expr::Mul(
//     Box::new(Expr::Number(3.0)),
//     Box::new(Expr::Number(4.0))
//   ))
// )

// 2. Evaluate AST
let result = eval_expr(&expr)?;
// Result: Ok(14.0)
```

## Testing

The test suite uses property-based testing to ensure correctness:

```bash
cargo test
```

Key test properties:
- Parser never panics on any input
- Round-trip: generate AST → convert to string → parse → evaluate matches direct evaluation
- Mathematical properties hold (commutativity, associativity, etc.)

## Project Structure

```
compute-mcp/
├── Cargo.toml
├── src/
│   ├── lib.rs              # Parser and evaluator
│   ├── compute.pest        # Grammar definition (16 lines)
│   └── bin/
│       └── stdio_direct.rs # MCP server
└── tests/
    └── tests.rs            # Property and integration tests
```

## Dependencies

- `pest` & `pest_derive` - Parser generator
- `mcpr` - MCP protocol implementation  
- `serde` & `serde_json` - JSON serialization
- `proptest` - Property-based testing (dev dependency)

## License

MIT