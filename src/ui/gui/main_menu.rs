use async_channel::Sender;
use async_std::task::block_on;
use dioxus::prelude::*;

use crate::ui::{UiInput, gui::common::ControlPanelStyle};

#[component]
pub fn MainMenu() -> Element {
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
        button {
            class: "torpedo-button",
            style: "{buttons_display_style}",
            onclick: move |_| {
                let sender = use_context::<Sender<UiInput>>();
                block_on(sender.send(UiInput::Exit)).expect("");
            },
            "Exit"
        }
        ControlPanelStyle {
            style: "margin: 3em auto; {details_display_style}",

            form {
                onsubmit: move |_| {},
                h2 {
                    style: "margin: 0 auto; font-size: 2em",
                    "{details_title}"
                }
                div {
                    class: "form-inputs",
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
                }

                div {
                    style: "margin: 0 auto;",
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
                            let sender = use_context::<Sender<UiInput>>();
                            block_on(sender.send(if details_title().to_lowercase().contains("create") {
                                UiInput::HostGame {
                                    addr: url(),
                                    passwd: passwd(),
                                }
                            } else {
                                UiInput::JoinGame {
                                    addr: url(),
                                    passwd: passwd(),
                                }
                            })).expect("");
                        },
                        "continue"
                    }
                }
            }
        }
    }
}
