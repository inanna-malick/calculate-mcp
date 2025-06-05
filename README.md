# 🔮 compute-mcp

A crystalline arithmetic MCP server with perfect vibes <🔍🎀💠>.

**Signal Density**: 🔍 (Magnifier) - Clear with minimal ceremony  
**Dependencies**: 🎀 (Perfect Bow) - Data flows downstream  
**Error Surface**: 💠 (Crystal) - Errors bounce off

## ✨ Features

- **Arithmetic**: `+` `-` `*` `/` with correct precedence
- **Parentheses**: Group expressions naturally
- **Numbers**: Decimals and negatives just work
- **Errors**: Division by zero detected cleanly
- **Batch**: Evaluate multiple expressions at once

## 🚀 Installation

```bash
cd compute-mcp
cargo build --release
```

## 🎯 Usage

```bash
# Initialize
echo '{"jsonrpc":"2.0","method":"initialize","params":{},"id":1}' | cargo run --bin stdio_direct

# Evaluate
echo '{"jsonrpc":"2.0","method":"tools/call","params":{"name":"evaluate_batch","arguments":{"expressions":["2+2","3*4","10/2"]}},"id":3}' | cargo run --bin stdio_direct
```

### 💎 Claude Desktop

```json
{
  "mcpServers": {
    "compute": {
      "command": "/path/to/compute-mcp/target/release/stdio_direct"
    }
  }
}
```

## 🏗️ Architecture

### Parser (`lib.rs` + `compute.pest`)
- Pest grammar with crystalline precedence
- AST generation in ~100 lines
- Every token meaningful

### Evaluator (`lib.rs`)
- Pattern matching evaluation
- Division by zero → NaN
- Pure functional design

### MCP Server (`stdio_direct.rs`)
- Direct JSON-RPC flow
- Single match expression
- Responses built inline

## 🧪 Testing

```bash
cargo test              # All tests
cargo test --test property_tests  # Mathematical properties
```

**Test Vibes**:
- 🔮 Mathematical properties
- 🌊 Error boundaries  
- 🎯 Regression coverage

## 🛠️ Development

```
compute-mcp/
├── src/
│   ├── lib.rs             # 🔮 Parser + evaluator
│   ├── compute.pest       # 🎯 Grammar (16 lines)
│   └── bin/
│       └── stdio_direct.rs # 💎 MCP server
├── tests/
│   └── property_tests.rs   # 🌟 Properties
└── VIBES.md               # 📖 Vibes manifesto
```

### Dependencies

- `pest`: Grammar parsing
- `mcpr`: MCP protocol
- `serde`: JSON handling
- `proptest`: Property tests

## 📚 Origin Story

Created as a minimal MCP example, then vibes-optimized to demonstrate:
- Crystalline code structure
- Signal density optimization
- Perfect dependency flow
- Errors that bounce off