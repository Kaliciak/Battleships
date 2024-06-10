use async_channel::{Receiver, Sender};
use async_std::task::block_on;
use dioxus::prelude::*;
use dioxus_desktop::*;

use crate::{
    logic::GameState,
    model::{Direction, IncompleteBoard, Ship},
    ui::{self, UiInput, UiMessage},
    utils::log::Logger,
};

mod main_menu;
mod lobby;
mod boards;
mod common;

pub static ASSETS_DIR: &str = "assets";
pub static GAME_TITLE: &str = "Battleships";

pub fn run_gui(receiver: Receiver<UiMessage>, sender: Sender<UiInput>) {
    let window_config = WindowBuilder::new()
        .with_maximized(true)
        .with_title("Battleships")
        ;
    let config = Config::new().with_window(window_config);

    LaunchBuilder::desktop()
        .with_cfg(config)
        .with_context_provider(move || Box::new(sender.clone()))
        .with_context_provider(move || Box::new(receiver.clone()))
        .launch(App);
}

#[derive(Clone)]
enum GameScreenType {
    MainMenu,
    Lobby,
    Boards,
}

#[component]
fn App() -> Element {
    use_context_provider(|| Signal::new(GameScreenType::MainMenu));
    use_context_provider(|| Signal::new(Vec::<String>::new()));
    use_context_provider(|| Signal::new(IncompleteBoard(vec![])));
    use_context_provider(|| Signal::<Option<GameState>>::new(None));
    use_coroutine(|_: UnboundedReceiver<String>| {
        let mut screen_type = use_context::<Signal<GameScreenType>>();
        let mut receiver = use_context::<Receiver<UiMessage>>();
        let mut logs = use_context::<Signal<Vec<String>>>();
        let mut inc_board = use_context::<Signal<IncompleteBoard>>();
        let mut versus_state = use_context::<Signal<Option<GameState>>>();
        async move {
            loop {
                match receiver.recv().await.expect("") {
                    UiMessage::MainScreen => { screen_type.set(GameScreenType::MainMenu) }
                    UiMessage::Lobby => { screen_type.set(GameScreenType::Lobby) }
                    UiMessage::Log(s) => { logs.push(s) }
                    UiMessage::BoardConstruction(board) => { inc_board.set(board) }
                    UiMessage::PrintGameState(state) => {
                        screen_type.set(GameScreenType::Boards);
                        versus_state.set(Some(state));
                    }
                    UiMessage::Exit => { window().close() }
                }
            }
        }
    });

    rsx! {
        link { rel: "stylesheet", href: "{ASSETS_DIR}/style.css" }
        GameScreen {}
        LogsScreen {}
    }
}

#[component]
fn GameScreen() -> Element {
    let screen_type = use_context::<Signal<GameScreenType>>();

    match screen_type() {
        GameScreenType::MainMenu => rsx! { crate::gui::main_menu::MainMenu {} },
        GameScreenType::Lobby => rsx! { crate::gui::lobby::Lobby {} },
        GameScreenType::Boards => rsx! { crate::gui::boards::Boards {} },
    }
}

#[component]
fn LogsScreen() -> Element {
    let logs = use_context::<Signal<Vec<String>>>();
    rsx! {
        div {
            class: "old-screen",
            for log in logs().iter().rev() {
                p { "{log}" }
            }
        }
    }
}
