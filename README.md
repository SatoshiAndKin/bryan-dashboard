# Bryan Dashboard

A web-based spreadsheet application with built-in Ethereum/Web3 capabilities, built with [Dioxus 0.7](https://dioxuslabs.com/learn/0.7) and compiled to WebAssembly.

Think **Apple Numbers meets Etherscan** — multiple named tables per sheet, real-time block data, on-chain balance lookups, and a formula engine that speaks both `SUM` and `BLOCK_NUMBER`.

## What Is This?

Bryan Dashboard is a personal productivity tool that combines a full-featured spreadsheet with live Ethereum data. You can:

- Build spreadsheets with multiple sheets and tables (like Apple Numbers, not just a single grid)
- Reference cells across tables and sheets with `=Table 1::A1` or `=Sheet 1::Table 1::A1`
- Use named column/row headers in formulas (e.g., `=Price` instead of `=B2`)
- Connect to any Ethereum RPC (WebSocket or HTTP) and pull live block data directly into cells
- Query ETH balances with `=ETH_BALANCE("0x...")` — values update automatically
- Export/import your workbook as JSON for backup or sharing

Everything runs in the browser as WASM. Data is persisted to `localStorage` with autosave.

## Features

- **Multi-sheet, multi-table spreadsheets** — Apple Numbers-style layout with named tables on a scrollable canvas
- **Formula engine** — arithmetic, cell references (`A1`, `$A$1`), ranges (`A1:B5`), cross-table refs (`Table 1::A1`), cross-sheet refs (`Sheet 1::Table 1::A1`), named column/row refs
- **Built-in functions** — `SUM`, `AVG`/`AVERAGE`, plus Ethereum-specific formulas
- **Ethereum integration** — connect via WebSocket or HTTP RPC to get real-time block data
  - `=BLOCK_NUMBER()` / `=BLOCK_HASH()` / `=BLOCK_TIMESTAMP()` / `=BASE_FEE()`
  - `=ETH_BALANCE("0x...")` — live ETH balance in ETH
- **Header rows/columns** — configurable headers and footers with named columns and rows usable in formulas
- **Clipboard** — Ctrl+C / Ctrl+V / Ctrl+X with reference shifting on paste
- **Drag & drop** — move cells between positions, formulas auto-update
- **Pin references** — `$A1`, `A$1`, `$A$1` to lock column/row during copy
- **Persistence** — autosave to `localStorage`, plus JSON export/import
- **Dark theme** — navy/blue palette designed for extended use
- **Starfield background** — animated canvas starfield behind the spreadsheet
- **Buddy character** — a little ASCII `(o_o)` that wanders the screen and flees your cursor
- **EIP-6963 support** — multi-injected wallet provider discovery

## Getting Started

### Prerequisites

- [Rust](https://rustup.rs/) (stable)
- [Dioxus CLI](https://dioxuslabs.com/learn/0.7)

```bash
curl -sSL http://dioxus.dev/install.sh | sh
```

### Development

```bash
dx serve
```

This starts a local dev server with hot-reload. The app runs entirely in the browser as WASM.

### Building for Production

```bash
dx build --release
```

### Running Tests

```bash
cargo test
```

## Architecture

```
src/
  main.rs              — app entry point
  eth/
    mod.rs             — BlockHead parsing from JSON-RPC
    eip6963.rs         — EIP-6963 multi-injected provider discovery
  model/
    workbook.rs        — WorkbookState (sheets, migration v1→v2)
    sheet.rs           — Sheet (multiple tables per sheet)
    table.rs           — TableModel (cells, headers, row/col ops)
    cell.rs            — CellRef, CellValue, CellModel
    settings.rs        — AppSettings (RPC URL, Etherscan key)
  formula/
    lexer.rs           — tokenizer (supports spaces in table/sheet names)
    parser.rs          — recursive descent parser → AST
    ast.rs             — expression types (cell refs, named refs, cross-table, ranges)
    eval.rs            — evaluator (arithmetic, functions, Web3 lookups)
    graph.rs           — dependency graph + topological recalculation
    rewrite.rs         — formula ref rewriting on move/copy/delete (respects $ pinning)
    registry.rs        — function docs for the sidebar UI
  persistence/         — localStorage save/load, JSON export/import
  ui/
    shell.rs           — root layout, keyboard handling, Ethereum connection
    grid.rs            — table grid rendering with headers
    cell_view.rs       — individual cell (display, edit, drag)
    tabs.rs            — sheet tab bar
    settings_pane.rs   — settings modal (RPC, Etherscan key)
    func_sidebar.rs    — formula function reference sidebar
    starfield.rs       — animated starfield background
    buddy.rs           — wandering ASCII buddy character
    confirm_modal.rs   — deletion confirmation dialog
```

## Formula Reference

| Function | Syntax | Description |
|---|---|---|
| `SUM` | `SUM(A1:A5)` | Sum of values in a range |
| `AVG` | `AVG(A1:A5)` | Average, ignoring empty cells |
| `BLOCK_NUMBER` | `BLOCK_NUMBER()` | Latest Ethereum block number |
| `BLOCK_HASH` | `BLOCK_HASH()` | Latest block hash |
| `BLOCK_TIMESTAMP` | `BLOCK_TIMESTAMP()` | Block timestamp (unix) |
| `BASE_FEE` | `BASE_FEE()` | Block base fee in wei |
| `ETH_BALANCE` | `ETH_BALANCE("0x...")` | ETH balance of an address |

### Cell References

| Type | Example | Description |
|---|---|---|
| Simple | `A1` | Column A, row 1 |
| Pinned col | `$A1` | Column locked during copy |
| Pinned row | `A$1` | Row locked during copy |
| Fully pinned | `$A$1` | Both locked during copy |
| Cross-table | `Table 1::A1` | Reference another table |
| Cross-sheet | `Sheet 1::Table 1::A1` | Reference another sheet's table |
| Named ref | `Price` | Reference by column/row header name |
| Range | `A1:B5` | Rectangular cell range |

## Settings

Open the Settings pane to configure:

- **Ethereum RPC URL** — WebSocket (`wss://...`) for real-time subscriptions, or HTTP (`https://...`) for polling
- **Poll interval** — how often to poll when using HTTP (default: 10s)
- **Etherscan v2 API Key** — for ABI fetching and contract verification

## Dioxus Quick Reference

This project uses [Dioxus 0.7](https://dioxuslabs.com/learn/0.7). Key differences from older versions:
- No `cx`, `Scope`, or `use_state` — use `use_signal` instead
- Components are `#[component] fn Name() -> Element`
- RSX uses `rsx! { ... }` syntax
- State is managed with `Signal<T>` and `use_memo`
