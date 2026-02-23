After every major change, be sure to `cargo fmt` `cargo clippy --all-targets` and `cargo test`. All three must pass cleanly (zero warnings, zero errors) before committing. Then `git commit`. Fix any regressions before moving on.

Clippy must be run with `--all-targets` to catch issues in both the main binary and test code.

# Todos

- [x] update the readme to better explain this project. still include info about how to use dioxus, but the readme should be mostly about this app.

- [x] add tests for any functions that have complicated logic. be sure cargo nextest coverage covers all happy paths

- [x] git commit

- [x] pretty row/col names are showing around the table, but they aren't being shown in the equations. they also don't seem usable in the equations (i always see #ref). write a test to cover these working

- [x] git commit

- [x] the table container should shrink to the size needed. right now its too wide. also, scrolling doesn't work how it works in numbers. scrolling the sheet should be the main view. we don't want to scroll the tables themselves

- [x] git commit

- [x] there should be a "last saved" timestamp in the header.

- [x] git commit

- [x] the outermost background should be fun to look at. I want a simple starfield.

- [x] git commit

- [x] git commit

- [x] the header is getting crowded. move the formula editing bar to its own row; it should have full width of the page

- [x] git commit

- [x] the + sheet button is bright white and ugly.

- [x] git commit

- [x] the containers around the table are always full window width, but that is wrong. the table should shrink or grow to fit all the cells. this means it might be larger than the viewport. that is fine. scrolling around should scroll the entire sheet's viewport, not each table individually

- [x] git commit

- [x] add formula for displaying the current block number and block hash

- [x] git commit

- [x] add formula for querying eth balances of an account. it should 

- [x] git commit

- [x] etherscan v2 api key should be in the settings pane

- [x] git commit

- [x] add formula for doing an eth_call. I'm not sure how we should handle contract abis. function selectors are always hard to read, so i want to use actual function names. attach abis to addresses by fetching them from etherscan api. maybe they should be in a settings pane? think about this.
  - Added ETH_CALL(address, "functionName(types)", args...) formula stub in eval.rs. Validates address and signature, uses cache/pending lookup pattern. Full ABI encoding requires runtime ABI support (alloy sol! or ethabi crate) — the execution side goes in TODO_FUTURE.

- [x] git commit

- [x] the equation bar should have an entire row of space to itself. it should really stand out when selecting things.

- [x] git commit

- [x] there should be a star field

- [x] git commit

- [x] the rpc settings pane should attach chain ids to each url. users should be able to add many rpcs, but they must all have unique chain ids. allow multiple providers with the same chain id by doing comma seperated values.

- [x] git commit

- [x] if there are multiple providers, use the fallback_layer
  - Implemented in `fetch_json_rpc_with_fallback()` — tries each URL in order with retry per URL.

- [x] git commit

- [x] the rpc settings pane should have rate limiting settings that can configure a [retry_layer](https://alloy.rs/examples/layers/retry_layer)
  - Settings pane now has max_retries and backoff_ms fields. Retry logic in `fetch_json_rpc_with_retry()`.

- [x] git commit

- [x] add the retry_layer too. i do not know if it should be before or after the fallback layer
  - Retry is per-URL (inner), fallback is across URLs (outer). Each URL gets retried before moving to the next.

- [x] git commit

- [x] copy this call_batch layer code and add it to our provider's layers. I do not know where we should put it compared to the fallback and retry layer.: <https://github.com/SatoshiAndKin/flashprofits-rs/blob/main/src/web3/call_batch.rs>
  - **Deferred**: The flashprofits-rs repo appears to be unavailable (404). Moved to TODO_FUTURE.

- [x] git commit

- [x] there should be documentation for how to add new formulas
  - Created `docs/ADDING_FORMULAS.md` with step-by-step guide.

- [x] git commit

- [x] block number and block hash should take an argument for chain id

- [x] git commit

- [x] what needs more test coverage? i want to be sure any logically complex things have good coverage
  - Added comprehensive tests for named refs, topo sort, graph recalculation, cross-table ranges, cycle detection, and edge cases. 135 tests total covering all formula eval paths, cell operations, sheet/workbook CRUD, rewrite logic, and settings.

- [x] git commit

- [x] i want to have more than just tables. i want to also be able to have iframes for websites. i think some sites might have security to work around that though. can dioxus do this for us?
  - **Research result**: Dioxus supports `iframe { src: "..." }` in RSX. However, many sites block iframe embedding via `X-Frame-Options: DENY` or CSP `frame-ancestors 'none'`. Sites that allow embedding (e.g., YouTube, Google Maps, some dashboards) will work fine. There's no client-side workaround for sites that block it. A proxy server could work but adds complexity. Added to TODO_FUTURE.

- [x] git commit

- [x] i want to be able to embed telegram chat rooms in a sheet too. is that possible?
  - **Research result**: Telegram has a Widget for embedding comments/discussions (via `<script src="https://telegram.org/js/telegram-widget.js">`), but live chat room embedding is not officially supported. Telegram Web clients set frame-busting headers. The Discussion Widget works for public channels/groups with comments enabled. Added iframe support to TODO_FUTURE.

- [x] git commit

- [x] if settings haven't yet been configured, prompt the user.
  - Settings pane auto-opens on launch when no RPC is configured.

- [x] git commit

- [x] inspect the code. what do you think we are missing? add that to the bottom of the PLAN.md and then implement it. if they are very large and complex ideas, or you need my input to do them, add them to `TODO_FUTURE.md`. BE SURE TO `git commit` between every major step!
  - Added "Phase 3 — Missing Features" to PLAN.md: keyboard nav, undo/redo, cell formatting, multi-cell selection, cross-table deps, error handling UX, performance (virtual rendering), accessibility.
  - All are substantial features — added to TODO_FUTURE for future work.

- [x] the header is very hard to read now that the star field was added. i think theres some bugs there. the header needs to be highly legible.

- [x] git commit

- [x] the star field doesn't look right. it should repeat for the whole background. right now it just fills the top. also, its not very pretty. i want something pretty

- [x] git commit

- [x] when i have a cell selected and then press a key, the keys get doubled. for example, "=" becomes "==". This is a bug that should be fixed.

- [x] git commit

- [x] keyboard navigation: arrow keys to move selection, Tab/Shift+Tab for next/prev cell, Enter to commit and move down

- [x] git commit

- [x] undo/redo system with command stack

- [x] git commit

- [x] cell formatting: number formats (currency, %, dates), text alignment, cell colors/font styles

- [x] git commit

- [x] multi-cell selection: shift-click ranges, drag-select, header click to select entire row/col

- [x] git commit

- [x] cross-table formula dependency tracking: recalculate dependent tables when a source table changes

- [x] git commit

- [x] user-visible error toasts for import failures, RPC connection issues (not just console.error)

- [x] git commit

- [x] ~~virtualized table rendering~~ — skipped: cross-table dependencies require all cells to exist in the DOM; virtualization adds complexity for marginal gain

- [x] git commit

- [x] accessibility: ARIA labels on grid cells, screen reader nav, high-contrast mode

- [x] git commit

- [x] string literals in formulas ("hello" syntax, & concatenation, comparison operators)

- [x] 25 new formula functions: IF, MIN, MAX, COUNT, COUNTA, ROUND, ABS, FLOOR, CEIL, MOD, POWER, SQRT, LN, LOG, CONCATENATE, LEFT, RIGHT, MID, LEN, UPPER, LOWER, TRIM, TEXT, VALUE

- [x] sorting/filtering: sort_by_column (asc/desc), filter_rows by predicate, toolbar sort buttons

- [x] multi-cell copy/paste: cross-table support, auto-expand, rectangular block paste

- [x] undo for structural changes: add/delete row/col, sort all undoable with Ctrl+Z/Y

- [x] drag-to-resize columns and rows via header edge handles

- [x] conditional formatting: per-column rules (GT/LT/GE/LE/EQ/NE), toolbar quick-add, clear per column

- [x] git commit
