# How to Add New Formulas

This guide walks through adding a new formula function to Bryan Dashboard.

## Steps

### 1. Add the function to the evaluator (`src/formula/eval.rs`)

In the `eval_func` match block, add a new arm:

```rust
"MY_FUNC" => {
    if args.is_empty() {
        return CellValue::Error("#ARGS! MY_FUNC(arg1, ...)".to_string());
    }
    let val = evaluate(&args[0], ctx);
    // ... your logic here ...
    CellValue::Number(result)
}
```

Key patterns:
- Use `evaluate(&args[n], ctx)` to evaluate each argument
- Use `collect_values(args, ctx)` for range-aware argument collection (respects `A1:B5` ranges)
- Return `CellValue::Number`, `CellValue::Text`, or `CellValue::Error` as appropriate
- Access `ctx.block_head` for Ethereum block data
- Access `ctx.balance_cache` / `ctx.pending_lookups` for async data that needs fetching

### 2. Register the function in the sidebar (`src/formula/registry.rs`)

Add an entry to `BUILTIN_FUNCTIONS`:

```rust
FuncInfo {
    name: "MY_FUNC",
    syntax: "MY_FUNC(arg1, [arg2])",
    description: "What this function does.",
},
```

### 3. Add tests (`src/formula/eval.rs` test module)

```rust
#[test]
fn test_my_func() {
    let mut t = make_table(2, 1);
    t.set_cell_source(0, 0, "42".to_string());
    t.set_cell_source(0, 1, "=MY_FUNC(A1)".to_string());
    recalculate_table(&mut t);
    assert_eq!(t.cells[&(0, 1)].computed, CellValue::Number(expected));
}
```

### 4. Build and test

```bash
cargo fmt
cargo clippy
cargo test
```

## Architecture Notes

- **Parser** (`parser.rs`): Function calls are parsed when the parser sees `IDENT(`. Function names are uppercased automatically. You don't need to modify the parser.
- **Lexer** (`lexer.rs`): No changes needed. Identifiers and parentheses are already tokenized.
- **Dependency graph** (`graph.rs`): Only local `CellRef` dependencies are tracked for cycle detection. If your function reads cells, those deps are tracked automatically through the AST. Named refs are resolved at eval time.
- **AST** (`ast.rs`): `Expr::FuncCall(String, Vec<Expr>)` already handles arbitrary function calls.

## Async Functions (e.g., Web3 lookups)

For functions that need async data (network calls):

1. Check `ctx.balance_cache` (or a similar cache) for a cached result
2. If not cached, add the request key to `ctx.pending_lookups` and return `CellValue::Text("#LOADING...")`
3. The shell's `use_effect` will pick up pending lookups, fetch data, update the cache, and trigger a recalculation

See `ETH_BALANCE` in `eval.rs` for a complete example.

## Function Categories

- **Math/Stats**: SUM, AVG — operate on ranges via `collect_values`
- **Block data**: BLOCK_NUMBER, BLOCK_HASH, etc. — read from `ctx.block_head`
- **On-chain**: ETH_BALANCE, ETH_CALL — use cache + pending lookup pattern
