use async_std::task::block_on;

use crate::gui::Gui;

use super::{game_init, game_loop};

async fn run_logic_async(mut gui: impl Gui) {
    loop {
        gui.go_to_main_screen();
        match gui.receive_input().await {
            crate::gui::Input::HostGame { addr, passwd } => {
                if let Some(channel) = game_init::create_host(&addr, &passwd, &mut gui).await {
                    game_loop::game_loop(channel, &mut gui).await;
                }
            }
            crate::gui::Input::JoinGame { addr, passwd } => {
                if let Some(channel) = game_init::create_client(&addr, &passwd, &mut gui).await {
                    game_loop::game_loop(channel, &mut gui).await;
                }
            }
            crate::gui::Input::Exit => {
                return;
            }
            _ => {}
        }
    }
}

pub fn run_logic(gui: impl Gui) {
    block_on(run_logic_async(gui));
}
