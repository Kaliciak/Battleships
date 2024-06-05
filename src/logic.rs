use std::{cell::RefCell, io::Write};

use async_channel::Receiver;
use async_std::task::block_on;
use main::run_logic_async;
use rustyline_async::{Readline, SharedWriter};
use serde::{Deserialize, Serialize};

use crate::{
    crypto::proofs::BoardCorrectnessProof,
    gui::{self, GuiInput, GuiMessage},
    model::SHIP_SIZES,
    utils::{async_receiver::AsyncReceiver, log::Log, result::Res, threads::parallel},
    Direction,
};

mod board_creation;
mod game_loop;
pub mod main;

/// Possible message received from another player
#[derive(Debug, Serialize, Deserialize)]
pub enum GameMessage {
    BoardIsCorrect(BoardCorrectnessProof, [u8; 32]),
}

pub fn run_main_loop_with_cli() {
    struct Cli {
        reader: Readline,
        writer: RefCell<SharedWriter>,
        state: GuiMessage,
    }
    impl Log for Cli {
        fn log_message(&self, msg: &str) -> Res<()> {
            self.writer
                .borrow_mut()
                .write_all(format!("{}\n", msg).as_bytes())?;
            Ok(())
        }
    }

    impl Cli {
        async fn receive_input(&mut self) -> Res<gui::GuiInput> {
            loop {
                let input = self.reader.readline().await.unwrap();
                match input {
                    rustyline_async::ReadlineEvent::Line(line) => {
                        self.reader.add_history_entry(line.clone());
                        let words = line.split(' ').collect::<Vec<&str>>();
                        if words.len() < 3 {
                            self.log_message("Invalid command")?;
                            continue;
                        }
                        if words[0] == "create" {
                            return Ok(GuiInput::HostGame {
                                addr: words[1].to_owned(),
                                passwd: words[2].to_owned(),
                            });
                        }
                        if words[0] == "join" {
                            return Ok(GuiInput::JoinGame {
                                addr: words[1].to_owned(),
                                passwd: words[2].to_owned(),
                            });
                        }
                        if words[0] == "msg" {
                            return Ok(GuiInput::SendMessage(
                                words[1].to_owned(),
                                words[2..].join(" ").to_owned(),
                            ));
                        }
                        if words[0] == "put" {
                            if words.len() != 4 {
                                self.log_message("Invalid command")?;
                                continue;
                            }
                            if let GuiMessage::BoardConstruction(inc_board) = &self.state {
                                return Ok(GuiInput::PutShip(crate::Ship {
                                    x: words[1].parse().unwrap(),
                                    y: words[2].parse().unwrap(),
                                    size: SHIP_SIZES[inc_board.0.len()],
                                    direction: {
                                        if words[3] == "down" {
                                            Direction::VERTICAL
                                        } else {
                                            Direction::HORIZONTAL
                                        }
                                    },
                                }));
                            }
                        }
                        self.log_message("Invalid command")?;
                    }
                    rustyline_async::ReadlineEvent::Eof => {
                        return Ok(gui::GuiInput::Exit);
                    }
                    rustyline_async::ReadlineEvent::Interrupted => return Ok(gui::GuiInput::Esc),
                }
            }
        }
        fn draw(&mut self) {
            match &self.state {
                GuiMessage::MainScreen => {
                    self.log_message("\n\nWitamy w grze w statki!\n\n   create address:port password => create game\n   join address:port password => join game\n   msg name info => send msg to the second player\n   Ctrl-C => Interrupt\n   Ctrl-D => Exit\n").unwrap();
                }
                GuiMessage::Lobby => {
                    self.log_message("\n\nLobby\n").unwrap();
                }
                GuiMessage::BoardConstruction(board) => {
                    self.log_message(&format!("{:#?}", board)).unwrap();
                }
                _ => {}
            }
        }
    }

    let (reader, writer) = Readline::new("> ".to_string()).unwrap();

    let (mut s_input, mut r_input) = async_channel::unbounded::<GuiMessage>();
    let (s_output, r_output) = async_channel::unbounded::<GuiInput>();

    async fn get_input_from(input: &mut Receiver<GuiMessage>) -> Res<GuiMessage> {
        Ok(input.recv().await?)
    }
    let mut cli = Cli {
        reader,
        writer: RefCell::new(writer),
        state: GuiMessage::MainScreen,
    };

    let f = async move {
        loop {
            match parallel(cli.receive_input(), get_input_from(&mut r_input)).await? {
                futures::future::Either::Left(input) => {
                    s_output.send(input).await.unwrap();
                }
                futures::future::Either::Right(state) => {
                    if let GuiMessage::Log(m) = state {
                        cli.log_message(&m).unwrap();
                    } else {
                        cli.state = state;
                        cli.draw();
                    }
                }
            }
        }
    };

    block_on(parallel::<(), ()>(
        f,
        run_logic_async(&mut AsyncReceiver(r_output), &mut s_input),
    ))
    .unwrap();
}
