use dioxus::prelude::*;

use crate::{
    gui::{ASSETS_DIR, GameScreenType},
    ui::UiInput,
};

#[component]
pub fn MainMenu() -> Element {
    let mut screen_type = consume_context::<Signal<GameScreenType>>();
    let mut message = use_context::<Signal<Option<UiInput>>>();

    let mut details_display_style = use_signal(|| "display: none".to_string());
    let mut buttons_display_style = use_signal(|| "".to_string());
    let mut details_title = use_signal(|| "".to_string());

    let mut url = use_signal(|| "".to_string());
    let mut passwd = use_signal(|| "".to_string());


    rsx! {
        h1 { class: "main-title", "Battleships" }
        button {
            class: "torpedo-button",
            style: "{buttons_display_style}",
            onclick: move |_| {
                *details_display_style.write() = "".to_string();
                *buttons_display_style.write() = "display: none".to_string();
                *details_title.write() = "Create room".to_string();
            },
            "Create room"
        }
        button {
            class: "torpedo-button",
            style: "{buttons_display_style}",
            onclick: move |_| {
                *details_display_style.write() = "".to_string();
                *buttons_display_style.write() = "display: none".to_string();
                *details_title.write() = "Join room".to_string();
            },
            "Join room"
        }
        ControlPanelStyledForm {
            style: "{details_display_style}",

            h2 {
                style: "margin: 0 auto; grid-column: 1/3; font-size: 2em",
                "{details_title}"
            }
            label {
                r#for: "url-input",
                "URL"
            }
            input {
                id: "url-input",
                r#type: "url",
                value: "{url}",
                required: true,
                oninput: move |event| url.set(event.value())
            }
    
            label {
                r#for: "pwd-input",
                "password"
            }
            input {
                id: "pwd-input",
                value: "{passwd}",
                required: true,
                oninput: move |event| passwd.set(event.value())
            }

            div {
                style: "margin: 0 auto; grid-column: 1/3",
                button {
                    class: "abort-button",
                    style: "margin: 0 1em 0 auto; display: inline",
                    onclick: move |_| {
                        *details_display_style.write() = "display: none".to_string();
                        *buttons_display_style.write() = "".to_string();
                        *url.write() = "".to_string();
                        *passwd.write() = "".to_string();
                    },
                    "cancel"
                }
                button {
                    class: "ok-button",
                    style: "display: inline",
                    onclick: move |_| {
                        *message.write() = if "{title}".to_lowercase().contains("host") {
                            Some(UiInput::HostGame {
                                addr: url(),
                                passwd: passwd(),
                            })
                        } else {
                            Some(UiInput::JoinGame {
                                addr: url(),
                                passwd: passwd(),
                            })
                        };
                        *screen_type.write() = GameScreenType::Lobby;
                    },
                    "continue"
                }
            }
        }
    }
}

#[component]
fn ControlPanelStyledForm(style: String, children: Element) -> Element {
    let mut decorations_styles = Vec::<String>::new();
    for i in 0..4 {
        let margin = if i % 2 == 0 {
            "5px"
        } else {
            "5px 5px 5px auto"
        };
        let grid_column = (i % 2) + 1;
        let grid_row = if i < 2 {
            "grid-row: 1"
        } else {
            ""
        };

        decorations_styles.push(
            format!("margin: {margin}; grid-column: {grid_column}; {grid_row}").to_string()
        );
    }

    rsx! {
        form {
            class: "control-panel-form",
            style: "{style}",
            onsubmit: move |_| {},

            {children}

            for style in decorations_styles {
                img {
                    aria_hidden: true,
                    style: "{style}",
                    src: "{ASSETS_DIR}/screw.svg"
                }
            }
        }
    }
}
