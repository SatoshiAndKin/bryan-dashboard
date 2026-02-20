- [ ] i want to view farcaster mini apps. will need farcaster-sdk's context for that to work. we'll have to build a whole farcaster host for that to work.

- [ ] i want to use neynar apis to show a list of casts for /channel or a user's follower's (anything the user's followers like/cast/quote gets included WITH reply bumping). then people can use this dashboard 

- [ ] i want to be able to use the browser's wallet and provider instead of one in the settings

- [ ] there should be a little clippy-like character in the corner. it should run around the screen and try to stay away from the cursor and prefer to stay on top of empty cells 

- [ ] we need an alloy provider layer that sends transactions to all of the private mempools in parallel. it should return as soon as it gets a single successful response, but it should be sure to send to ALL private mempools. 

- [ ] should this code be a part of flashprofits-rs? or how should we integrate price/path finding with that? i think maybe having a openapi swagger connector that lets people build forms with no-code

- [ ] import/export should be a lot more advanced. i want to be able to export just pieces. and importing should merge things. doing this well

- [ ] window.ethereum works for using a wallet provider, but it is fragile. there is a multi-provider ERC that we should be using instead. i really dislike javascript, so we should keep as much of this logic in our rust/alloy code as possible

- [ ] have an ai chatbot table that you can do whatever with. have LLM(query) and it puts a number into the cell. this will need smart caching. maybe a custom polling interval for each cell.

- [ ] i probably need some MCP servers on factory.ai. i think claude has some memory stuff that it saves in the repo that keeps it from forgetting so much. i want to teach it best practices and have it actually remember. so my "rust engineer" skill will just use dioxus/alloy/serde/tokio/tracing/dotenvy/envy without me having to tell it. i feel like i need a library of best practices for everything. then tell the agent "go through the library and build a skill relevant to the task at hand. then do the task"
    - like, right now i have AGENTS.md that describes a single agent that knows dioxus. but i need one that knows alloy, too. and i think they should be one that knows multiple topics. i've seen lots of people makign a bunch of hyper specialized agents. but i find most of the code i write to be gluing multiple projects together. you need cross domain knowledge for that.

- [ ] whenever the block updates, all the relevant cells need to update. how can we leverage dioxus signals for this?

- [ ] how should getting prices/routes from flashprofits-rs work?

- [ ] is it possible to attach `window.ethereum` to the alloy provider? I think maybe not. discuss this.

- [ ] copy call_batch layer from flashprofits-rs once the repo is available again. The layer would batch multiple JSON-RPC calls into a single request. Should go after retry but before fallback in the layer stack.

- [ ] add iframe widget support to sheets — allow embedding websites alongside tables. Many sites block iframe embedding via X-Frame-Options, so this works best for sites that explicitly allow it (YouTube, Google Maps, dashboards with CORS). Could add a proxy server option later.

- [ ] add Telegram Discussion Widget embedding — use telegram-widget.js for public channels with comments. Live chat room embedding is not officially supported by Telegram.

- [ ] make sure it compiles with `~/.cargo/bin/dx build`