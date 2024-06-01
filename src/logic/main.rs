use async_std::task::block_on;

use crate::{
    gui::{get_gui_input, Gui},
    utils::input::InputFilter,
};

use super::{game_init, game_loop};

async fn main_loop(mut filter: InputFilter, mut gui: impl Gui) {
    loop {
        gui.go_to_main_screen();
        let (main_loop, mut main_filter) = filter.advance();

        main_loop
            .run(async {
                match get_gui_input(&mut filter, &mut gui).await {
                    crate::gui::Input::HostGame { addr, passwd } => {
                        if let Some(channel) =
                            game_init::create_host(&addr, &passwd, &mut main_filter, &mut gui).await
                        {
                            game_loop::game_loop(&mut main_filter, channel, &mut gui).await;
                        }
                    }
                    crate::gui::Input::JoinGame { addr, passwd } => {
                        if let Some(channel) =
                            game_init::create_client(&addr, &passwd, &mut main_filter, &mut gui)
                                .await
                        {
                            game_loop::game_loop(&mut main_filter, channel, &mut gui).await;
                        }
                    }
                    _ => {}
                }
            })
            .await;
    }
}

async fn run_logic_async(mut gui: impl Gui) {
    let (main, filter) = InputFilter::new(gui.get_logger());

    main.run(main_loop(filter, gui)).await;
}

pub fn run_logic(gui: impl Gui) {
    block_on(run_logic_async(gui));
}
