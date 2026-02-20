# Bryan Dashboard

A web-based spreadsheet application with built-in Ethereum/Web3 capabilities, built with [Dioxus 0.7](https://dioxuslabs.com/learn/0.7) and compiled to WebAssembly.

Think Apple Numbers meets Etherscan — multiple named tables per sheet, real-time block data, on-chain balance lookups, and a formula engine that speaks both `SUM` and `BLOCK_NUMBER`.

## Features

- **Multi-sheet, multi-table spreadsheets** — Apple Numbers-style layout with named tables on a scrollable canvas
- **Formula engine** — arithmetic, cell references (`A1`, `$A$1`), ranges (`A1:B5`), cross-table refs (`Table 1::A1`), cross-sheet refs (`Sheet 1::Table 1::A1`)
- **Built-in functions** — `SUM`, `AVG`/`AVERAGE`, plus Ethereum-specific formulas
- **Ethereum integration** — connect via WebSocket or HTTP RPC to get real-time block data
  - `=BLOCK_NUMBER()` / `=BLOCK_HASH()` / `=BLOCK_TIMESTAMP()` / `=BASE_FEE()`
  - `=ETH_BALANCE("0x...")` — live ETH balance in ETH
- **Header rows/columns** — configurable headers and footers with named columns and rows
- **Clipboard** — Ctrl+C / Ctrl+V / Ctrl+X with reference shifting on paste
- **Drag & drop** — move cells between positions, formulas auto-update
- **Persistence** — autosave to `localStorage`, plus JSON export/import
- **Dark theme** — navy/blue palette designed for extended use
- **Starfield background** — animated canvas starfield behind the spreadsheet
- **Buddy character** — a little ASCII `(o_o)` that wanders the screen

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
cargo nextest run
# or
cargo test
```

## Architecture

```
src/
  main.rs              — app entry point
  eth/                 — Ethereum block parsing
  model/
    workbook.rs        — WorkbookState (sheets, migration)
    sheet.rs           — Sheet (multiple tables)
    table.rs           — TableModel (cells, headers, operations)
    cell.rs            — CellRef, CellValue, CellModel
    settings.rs        — AppSettings (RPC URL, Etherscan key)
  formula/
    lexer.rs           — tokenizer
    parser.rs          — recursive descent parser
    ast.rs             — AST types
    eval.rs            — evaluator (including Web3 functions)
    graph.rs           — dependency graph + topological recalculation
    rewrite.rs         — formula ref rewriting (move, copy, delete)
    registry.rs        — function documentation for the sidebar
  persistence/         — localStorage save/load, export/import
  ui/
    shell.rs           — root component, keyboard handling, Ethereum connection
    grid.rs            — table grid rendering
    cell_view.rs       — individual cell component
    tabs.rs            — sheet tab bar
    settings_pane.rs   — settings modal
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

- `A1` — simple reference
- `$A1` — pinned column
- `A$1` — pinned row
- `$A$1` — fully pinned
- `Table 1::A1` — cross-table reference
- `Sheet 1::Table 1::A1` — cross-sheet reference

## Settings

Open the Settings pane to configure:

- **Ethereum RPC URL** — WebSocket (`wss://...`) for real-time subscriptions, or HTTP (`https://...`) for polling
- **Poll interval** — how often to poll when using HTTP (default: 10s)
- **Etherscan v2 API Key** — for future ABI fetching and contract verification
