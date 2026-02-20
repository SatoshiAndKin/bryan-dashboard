use dioxus::prelude::*;

mod formula;
mod model;
mod persistence;
mod ui;

use ui::shell::WorkbookShell;

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Stylesheet { href: MAIN_CSS }
        WorkbookShell {}
    }
}
