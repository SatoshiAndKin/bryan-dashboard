# Bryan Dashboard — Spreadsheet App Implementation Plan

## Overview
A Dioxus 0.7 web-only spreadsheet app (Numbers/Excel-like) with multiple named tables,
resizable rows/columns, double-click in-place cell editing, and a formula DSL.

## Phase 1 — MVP
- Multiple named tables with resizable rows/cols
- Double-click in-place cell editing
- Formula DSL: arithmetic + cell refs + ranges + `SUM`/`AVG`
- Dependency graph with recalculation and cycle detection
- Browser localStorage persistence with autosave

## Phase 2 — Web3
- Alloy provider integration
- `=ERC20_BALANCE("mainnet", "0xToken", "0xWallet")` formula function
- Async resolution with `#LOADING` / `#WEB3!` states
- Polling for live updates

## Architecture
```
src/
  main.rs              — launch + App root
  model/
    mod.rs             — re-exports
    workbook.rs        — WorkbookState, TableId
    table.rs           — TableModel
    cell.rs            — CellModel, CellValue, CellRef
  formula/
    mod.rs             — re-exports
    lexer.rs           — tokenizer
    parser.rs          — recursive descent parser -> AST
    ast.rs             — AST types
    eval.rs            — evaluator
    graph.rs           — dependency graph + recalc
  persistence/
    mod.rs             — localStorage load/save
  ui/
    mod.rs             — re-exports
    shell.rs           — WorkbookShell layout
    tabs.rs            — TableTabsPanel
    grid.rs            — SheetView + GridBody
    cell_view.rs       — CellView + CellEditor
```

## Formula Errors
- `#PARSE!` — syntax error
- `#REF!` — invalid cell reference
- `#DIV/0!` — division by zero
- `#CYCLE!` — circular dependency

## Persistence
- Key: `bd.workbook.v1`
- Autosave: 700ms debounce after mutations
- Versioned JSON payload
