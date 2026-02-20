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
- `#NO_RPC!` — no Ethereum RPC connected
- `#CHAIN!` — connected to wrong chain
- `#LOADING...` — async data being fetched
- `#NAME?` — unknown function name
- `#VALUE!` — invalid argument type
- `#ARGS!` — wrong number of arguments

## Persistence
- Key: `bd.workbook.v1`
- Autosave: 700ms debounce after mutations
- Versioned JSON payload

## Phase 3 — Missing Features (Identified via Code Inspection)

### Keyboard Navigation
- Arrow keys should move selection between cells (currently no arrow key handling)
- Tab should move to next cell, Shift+Tab to previous
- Enter should commit edit and move down

### Undo/Redo
- No undo/redo system exists. Should maintain a command stack for cell edits,
  row/col additions/deletions, and structural changes.

### Cell Formatting
- No number formatting (currency, percentage, date, decimal places)
- No text alignment (left/center/right)
- No cell background colors or font styles

### Multi-Cell Selection
- No shift-click or drag-select for ranges
- No "select all" in a column/row by clicking header
- Copy/paste only works for single cells

### Cross-Table Formula Dependencies
- The dependency graph only tracks intra-table refs. Cross-table refs are evaluated
  but not tracked for recalculation ordering. If Table 2 references Table 1, changing
  Table 1 won't automatically recalculate Table 2.

### Error Handling
- No user-visible error messages for import failures or RPC connection issues
  (currently only logged to browser console)

### Performance
- Large tables (100+ rows) may be slow due to full re-render on any cell change.
  Could benefit from virtualized rendering (only render visible cells).

### Accessibility
- No ARIA labels on grid cells
- No screen reader support for cell navigation
- No high-contrast mode
