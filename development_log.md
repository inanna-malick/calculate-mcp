# compute-mcp Development Log

## Project Overview
Development of a minimal arithmetic MCP (Model Context Protocol) tool for a blog post example, focusing on clean implementation and Rust best practices.

## Initial Implementation Phase

### Task 1: Fix Test Errors
**Initial Approach**: Started by fixing compilation warnings and test failures in the existing codebase.

**Key Issue**: Parser evaluation logic was incorrectly handling the parse tree structure - the code assumed nested structures where Pest actually produces flat sequences.

**Solution**: Rewrote the evaluation functions to properly traverse Pest's parse tree according to the grammar rules.

### Task 2: Compilation Warning
**Issue**: Unused import `Context` in stdio_direct.rs

**User Guidance**: "fix compile errors in stdio_direct.rs first" - directed me to prioritize the compilation warning before addressing test failures.

**Solution**: Removed the unused import.

## Debugging Methodology

### Parser Structure Analysis
**Initial Approach**: Created nested loops with `inner1`, `inner2`, etc. to debug the parser structure.

**User Correction**: "instead of inner1, inner 2, etc use recursive functions" - guided me to write cleaner, more idiomatic debugging code.

**Improved Solution**: Implemented a recursive `print_pair` function for cleaner parse tree visualization.

### Debug File Creation
**Initial Approach**: Started creating a `src/bin/debug_parser.rs` file for debugging.

**User Guidance**: "don't create bin files for debugging, write tests instead" - emphasized proper testing practices over ad-hoc debugging binaries.

**Solution**: Added debug tests within the test module to understand parser behavior.

## Refactoring Phase

### Strong Types and Idioms
**User Request**: "update to make sure strong types are in use, rust idioms, concise, readable, full property test coverage"

**Implementation**:
1. Created `Expression` type to prevent empty expressions
2. Replaced `anyhow` strings with custom `ComputeError` enum
3. Used `?` operator throughout for cleaner error handling
4. Expanded property tests from 10 to 22 tests

### Grammar Enhancement
**Issue Discovered**: Grammar didn't support unary minus on parenthesized expressions like `-(5 + 3)`

**Solution**: Added `unary` rule to the grammar and corresponding evaluation function.

## Key Learning Points

1. **Test-Driven Debugging**: User guidance to use tests instead of debug binaries led to better understanding and documentation of issues.

2. **Recursive Thinking**: The correction about using recursive functions instead of nested loops resulted in cleaner, more maintainable code.

3. **Incremental Development**: User's preference for `cargo check` during development instead of `cargo build --release` emphasized faster iteration cycles.

4. **Strong Types**: The push for strong types improved API safety and made invalid states unrepresentable.

## Final Outcome

- All 36 tests passing
- Type-safe API with `Expression`, `ComputeError`, and `EvaluationResult` types
- Idiomatic Rust code following community best practices
- Comprehensive property test coverage for mathematical properties
- Support for full arithmetic expressions including unary minus

The project successfully demonstrates building a minimal but robust MCP tool with proper error handling and type safety.