use std::io::Write;

use rustyline_async::{Readline, SharedWriter};
use serde::{Deserialize, Serialize};

use crate::{
    ui::{self, UI, Input, Logger},
    logic,
};

mod game_init;
mod game_loop;
pub mod main;

#[derive(Debug, Serialize, Deserialize)]
pub enum GameMessage {}

pub fn run_main_loop_with_cli() {
    #[derive(Clone)]
    struct CliLogger {
        writer: SharedWriter,
    }
    impl Logger for CliLogger {
        fn log_message(&mut self, msg: &str) {
            self.writer
                .write_all(format!("{}\n", msg).as_bytes())
                .expect("Error while logging");
        }
    }
    struct Cli {
        reader: Readline,
        writer: SharedWriter,
    }

    impl UI for Cli {
        async fn receive_input(&mut self) -> ui::Input {
            loop {
                let input = self.reader.readline().await.unwrap();
                match input {
                    rustyline_async::ReadlineEvent::Line(line) => {
                        self.reader.add_history_entry(line.clone());
                        let words = line.split(' ').collect::<Vec<&str>>();
                        if words.len() < 3 {
                            self.get_logger().log_message("Invalid command");
                            continue;
                        }
                        if words[0] == "create" {
                            return Input::HostGame {
                                addr: words[1].to_owned(),
                                passwd: words[2].to_owned(),
                            };
                        }
                        if words[0] == "join" {
                            return Input::JoinGame {
                                addr: words[1].to_owned(),
                                passwd: words[2].to_owned(),
                            };
                        }
                        if words[0] == "msg" {
                            return Input::SendMessage(
                                words[1].to_owned(),
                                words[2..].join(" ").to_owned(),
                            );
                        }
                        self.get_logger().log_message("Invalid command");
                    }
                    rustyline_async::ReadlineEvent::Eof => {
                        return ui::Input::Exit;
                    }
                    rustyline_async::ReadlineEvent::Interrupted => return ui::Input::Esc,
                }
            }
        }

        fn go_to_main_screen(&mut self) {
            self.get_logger().log_message("\n\nWitamy w grze w statki!\n\n   create address:port password => create game\n   join address:port password => join game\n   msg name info => send msg to the second player\n   Ctrl-C => Interrupt\n   Ctrl-D => Exit\n");
        }

        fn get_logger(&mut self) -> Box<dyn Logger> {
            Box::new(CliLogger {
                writer: self.writer.clone(),
            })
        }

        fn show_lobby(&mut self) {
            self.get_logger().log_message("\n\nLobby\n");
        }
    }

    let (reader, writer) = Readline::new("> ".to_string()).unwrap();

    logic::main::run_logic(Cli { reader, writer });
}
