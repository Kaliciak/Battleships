use dioxus::prelude::*;

use crate::gui::ASSETS_DIR;

#[component]
pub fn ControlPanelStyle(style: String, children: Element) -> Element {
    rsx! {
        div {
            class: "control-panel",
            style: "{style}",

            div {
                style: "position: relative; height: 36px",
                img {
                    style: "position: absolute; top: 5px; left: 5px",
                    src: "{ASSETS_DIR}/screw.svg"
                }
                img {
                    style: "position: absolute; top: 5px; right: 5px",
                    src: "{ASSETS_DIR}/screw.svg"
                }
            }

            div {
                style: "padding: 0 36px",
                {children}
            }

            div {
                style: "position: relative; height: 36px",
                img {
                    style: "position: absolute; bottom: 5px; left: 5px",
                    src: "{ASSETS_DIR}/screw.svg"
                }
                img {
                    style: "position: absolute; bottom: 5px; right: 5px",
                    src: "{ASSETS_DIR}/screw.svg"
                }
            }
        }
    }
}
