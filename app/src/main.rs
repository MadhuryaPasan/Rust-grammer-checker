use dioxus::prelude::*;

mod checker;
mod providers;
mod storage;
mod views;

const MAIN_CSS: Asset = asset!("/assets/styling/main.css");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

fn main() {
    #[cfg(feature = "desktop")]
    {
        use dioxus::desktop::{Config, WindowBuilder};
        use tao::dpi::LogicalSize;

        let window = WindowBuilder::new()
            .with_title("WordGlow")
            .with_inner_size(LogicalSize::new(1280.0, 800.0))
            .with_min_inner_size(LogicalSize::new(1200.0, 720.0));
        dioxus::LaunchBuilder::desktop()
            .with_cfg(Config::new().with_window(window))
            .launch(App);
    }

    #[cfg(not(feature = "desktop"))]
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }
        views::Home {}
    }
}
