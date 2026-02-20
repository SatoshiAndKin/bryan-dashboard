After every major change, be sure to `cargo fmt` `cargo clippy` and `git commit`. Then `cargo nextest` and fix any bugs.

# Todos

- [ ] add tests for any functions that have complicated logic. be sure cargo nextest coverage covers all happy paths

- [ ] window.ethereum works for using a wallet provider, but it is fragile. there is a multi-provider ERC that we should be using instead. i really dislike javascript, so we should keep as much of this logic in our rust/alloy code as possible

- [ ] pretty row/col names are showing around the table, but they aren't being shown in the equations. they also don't seem usable in the equations (i always see #ref). write a test to cover these working

- [ ] the table container should shrink to the size needed. right now its too wide. also, scrolling doesn't work how it works in numbers. scrolling the sheet should be the main view. we don't want to scroll the tables themselves

- [ ] there should be a "last saved" timestamp in the header.

- [ ] the outermost background should be fun to look at. I want a simple starfield.

- [ ] there should be a little clippy-like character in the corner. it should run around the screen and try to stay away from the cursor and prefer to stay on top of empty cells 

- [ ] the header is getting crowded. move the formula editing bar to its own row; it should have full width of the page

- [ ] the + sheet button is bright white and ugly.

- [ ] the containers around the table are always full window width, but that is wrong. the table should shrink or grow to fit all the cells. this means it might be larger than the viewport. that is fine. scrolling around should scroll the entire sheet's viewport, not each table individually

- [ ] add formula for displaying the current block number and block hash

- [ ] add formula for querying eth balances of an account. it should 

- [ ] etherscan v2 api key should be in the settings pane

- [ ] add formula for doing an eth_call. I'm not sure how we should handle contract abis. function selectors are always hard to read, so i want to use actual function names. attach abis to addresses by fetching them from etherscan api. maybe they should be in a settings pane? think about this.

- [ ] scan the code for anything else you think needs to be fixed and add them as more items in this TODO.md
