- [ ] i want to view farcaster mini apps. will need farcaster-sdk's context for that to work. we'll have to build a whole farcaster host for that to work.

- [ ] i want to use neynar apis to show a list of casts for /channel or a user's follower's (anything the user's followers like/cast/quote gets included WITH reply bumping). then people can use this dashboard 

- [ ] i want to be able to use the browser's wallet and provider instead of one in the settings

- [ ] there should be a little clippy-like character in the corner. it should run around the screen and try to stay away from the cursor and prefer to stay on top of empty cells 

- [ ] we need an alloy provider layer that sends transactions to all of the private mempools in parallel. it should return as soon as it gets a single successful response, but it should be sure to send to ALL private mempools. 

- [ ] should this code be a part of flashprofits-rs? or how should we integrate price/path finding with that? i think maybe having a openapi swagger connector that lets people build forms with no-code

- [ ] import/export should be a lot more advanced. i want to be able to export just pieces. and importing should merge things. doing this well

- [ ] window.ethereum works for using a wallet provider, but it is fragile. there is a multi-provider ERC that we should be using instead. i really dislike javascript, so we should keep as much of this logic in our rust/alloy code as possible
