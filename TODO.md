After every major change, be sure to `cargo fmt` `cargo clippy` and `git commit`. Then `cargo nextest` and fix any bugs. Once bugs are fixed, you can check off the TODO and commit again. Then, you can move on to the next todo.

# Todos

- [ ] update the readme to better explain this project. still include info about how to use dioxus, but the readme should be mostly about this app.

- [ ] add tests for any functions that have complicated logic. be sure cargo nextest coverage covers all happy paths

- [ ] git commit

- [ ] pretty row/col names are showing around the table, but they aren't being shown in the equations. they also don't seem usable in the equations (i always see #ref). write a test to cover these working

- [ ] git commit

- [ ] the table container should shrink to the size needed. right now its too wide. also, scrolling doesn't work how it works in numbers. scrolling the sheet should be the main view. we don't want to scroll the tables themselves

- [ ] git commit

- [ ] there should be a "last saved" timestamp in the header.

- [ ] git commit

- [ ] the outermost background should be fun to look at. I want a simple starfield.

- [ ] git commit

- [ ] git commit

- [ ] the header is getting crowded. move the formula editing bar to its own row; it should have full width of the page

- [ ] git commit

- [ ] the + sheet button is bright white and ugly.

- [ ] git commit

- [ ] the containers around the table are always full window width, but that is wrong. the table should shrink or grow to fit all the cells. this means it might be larger than the viewport. that is fine. scrolling around should scroll the entire sheet's viewport, not each table individually

- [ ] git commit

- [ ] add formula for displaying the current block number and block hash

- [ ] git commit

- [ ] add formula for querying eth balances of an account. it should 

- [ ] git commit

- [ ] etherscan v2 api key should be in the settings pane

- [ ] git commit

- [ ] add formula for doing an eth_call. I'm not sure how we should handle contract abis. function selectors are always hard to read, so i want to use actual function names. attach abis to addresses by fetching them from etherscan api. maybe they should be in a settings pane? think about this.

- [ ] git commit

- [ ] the equation bar should have an entire row of space to itself. it should really stand out when selecting things.

- [ ] git commit

- [ ] there should be a star field

- [ ] git commit

- [ ] the rpc settings pane should attach chain ids to each url. users should be able to add many rpcs, but they must all have unique chain ids. allow multiple providers with the same chain id by doing comma seperated values.

- [ ] git commit

- [ ] if there are multiple providers, use the fallback_layer

- [ ] git commit

- [ ] the rpc settings pane should have rate limiting settings that can configure a [retry_layer](https://alloy.rs/examples/layers/retry_layer)

- [ ] git commit

- [ ] add the retry_layer too. i do not know if it should be before or after the fallback layer

- [ ] git commit

- [ ] copy this call_batch layer code and add it to our provider's layers. I do not know where we should put it compared to the fallback and retry layer.: <https://github.com/SatoshiAndKin/flashprofits-rs/blob/main/src/web3/call_batch.rs>

- [ ] git commit

- [ ] is it possible to attach `window.ethereum` to the alloy provider? I think maybe not. discuss this.

- [ ] git commit

- [ ] there should be documentation for how to add new formulas

- [ ] git commit

- [ ] block number and block hash should take an argument for chain id

- [ ] git commit

- [ ] what needs more test coverage? i want to be sure any logically complex things have good coverage

- [ ] git commit

- [ ] i want to have more than just tables. i want to also be able to have iframes for websites. i think some sites might have security to work around that though. can dioxus do this for us?

- [ ] git commit

- [ ] i want to be able to embed telegram chat rooms in a sheet too. is that possible?

- [ ] git commit

- [ ] if settings haven't yet been configured, prompt the user.

- [ ] git commit

- [ ] inspect the code. what do you think we are missing? add that to the bottom of the PLAN.md and then implement it. if they are very large and complex ideas, or you need my input to do them, add them to `TODO_FUTURE.md`. BE SURE TO `git commit` between every major step!
