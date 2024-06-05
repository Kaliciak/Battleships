use dioxus::prelude::*;
use dioxus_desktop::*;

use crate::{
    ui::{self, UI, Input, Logger},
    logic,
};

static ASSETS_DIR: &str = "assets";

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
    Board,
}

#[component]
fn App() -> Element {
    use_context_provider(|| Signal::new(GameScreenType::MainMenu));

    rsx! {
        link { rel: "stylesheet", href: "{ASSETS_DIR}/style.css" }
        GameScreen {}
    }
}

#[component]
fn GameScreen() -> Element {
    let screen_type = consume_context::<Signal<GameScreenType>>();

    match screen_type() {
        GameScreenType::MainMenu => rsx! { MainMenu {} },
        GameScreenType::Board => rsx! { Board {} },
    }
}

#[component]
fn MainMenu() -> Element {
    let mut screen_type = consume_context::<Signal<GameScreenType>>();
	let mut form_style = use_signal(|| "display: none");
	let mut buttons_style = use_signal(|| "");

    rsx! {
        h1 { class: "main-title", "Battleships" }
        button {
            class: "torpedo-button",
			style: "{buttons_style}",
            //onclick: move |_| { *screen_type.write() = GameScreenType::Board },
			onclick: move |_| { *form_style.write() = ""; *buttons_style.write() = "display: none" },
            "Create room"
        }
        button {
            class: "torpedo-button",
			style: "{buttons_style}",
            //onclick: move |_| { *screen_type.write() = GameScreenType::Board },
			onclick: move |_| { *form_style.write() = ""; *buttons_style.write() = "display: none" },
            "Join room"
        }
		ConnectionDetails {
			id: "connection-data-form",
			style: form_style
		}
    }
}

#[component]
fn ConnectionDetails(id: String, style: String) -> Element {
	let mut url = use_signal(|| "".to_string());
	let mut passwd = use_signal(|| "".to_string());

	rsx! {
		form {
			id: "{id}",
			style: "{style}",
			onsubmit: move |_| {},
			label {
				r#for: "url-input",
				"URL"
			}
			input {
				id: "url-input",
				r#type: "url",
				value: "{url}",
				oninput: move |event| url.set(event.value())
			}
	
			label {
				r#for: "pwd-input",
				"password"
			}
			input {
				id: "pwd-input",
				value: "{passwd}",
				oninput: move |event| passwd.set(event.value())
			}

			button {
				style: "margin: 0 auto; grid-column: 1/3",
				"continue"
			}

			img {
				aria_hidden: true,
				style: "margin: 5px; grid-column: 1; grid-row: 1",
				src: "assets/screw.svg"
			}
			img {
				aria_hidden: true,
				style: "margin: 5px 5px 5px auto; grid-column: 2; grid-row: 1",
				src: "assets/screw.svg"
			}
			img {
				aria_hidden: true,
				style: "margin: 5px; grid-column: 1;",
				src: "assets/screw.svg"
			}
			img {
				aria_hidden: true,
				style: "margin: 5px 5px 5px auto; grid-column: 2;",
				src: "assets/screw.svg"
			}
		}
	}
}

#[component]
fn Board() -> Element {
    rsx! {
        h1 { "Game board" }
    }
}
