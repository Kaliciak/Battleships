use dioxus::prelude::*;
use dioxus_desktop::*;

use crate::{
    logic,
    ui::{self, UiInput, UiMessage},
    utils::log::Logger,
};

mod main_menu;

pub static ASSETS_DIR: &str = "assets";
pub static GAME_TITLE: &str = "Battleships";

pub fn launch_gui() {
    let window_config = WindowBuilder::new()
        .with_maximized(true)
        .with_title("Battleships")
        ;
    let config = Config::new().with_window(window_config);

    LaunchBuilder::desktop().with_cfg(config).launch(App);
}

#[derive(Clone)]
enum GameScreenType {
    MainMenu,
    Lobby,
    Board,
}

#[component]
fn App() -> Element {
    use_context_provider(|| Signal::new(GameScreenType::MainMenu));
    use_context_provider(|| Signal::<Option::<UiInput>>::new(None));

    rsx! {
        link { rel: "stylesheet", href: "{ASSETS_DIR}/style.css" }
        GameScreen {}
    }
}

#[component]
fn GameScreen() -> Element {
    let screen_type = consume_context::<Signal<GameScreenType>>();

    match screen_type() {
        GameScreenType::MainMenu => rsx! { crate::gui::main_menu::MainMenu {} },
        GameScreenType::Lobby => rsx! { Lobby {} },
        GameScreenType::Board => rsx! { Board {} },
    }
}

#[component]
fn Lobby() -> Element {
    rsx! {
        h1 { "Lobby" }
    }
}

#[component]
fn Board() -> Element {
    rsx! {
        h1 { "Game board" }
    }
}
