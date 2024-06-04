use async_std::task::block_on;

use crate::ui::UI;

use super::{game_init, game_loop};

async fn run_logic_async(mut ui: impl UI) {
    loop {
        ui.go_to_main_screen();
        match ui.receive_input().await {
            crate::ui::Input::HostGame { addr, passwd } => {
                if let Some(channel) = game_init::create_host(&addr, &passwd, &mut ui).await {
                    game_loop::game_loop(channel, &mut ui).await;
                }
            }
            crate::ui::Input::JoinGame { addr, passwd } => {
                if let Some(channel) = game_init::create_client(&addr, &passwd, &mut ui).await {
                    game_loop::game_loop(channel, &mut ui).await;
                }
            }
            crate::ui::Input::Exit => {
                return;
            }
            _ => {}
        }
    }
}

pub fn run_logic(ui: impl UI) {
    block_on(run_logic_async(ui));
}
