use std::{cell::RefCell, io::Write};

use async_channel::Receiver;
use async_std::task::block_on;
use game_loop::Player;
use main::run_logic_async;
use rustyline_async::{Readline, SharedWriter};
use serde::{Deserialize, Serialize};

use crate::{
    circuit::{
        board_declaration_circuit::BoardDeclarationCircuit,
        field_declaration_circuit::FieldDeclarationCircuit,
    },
    crypto::proofs::CorrectnessProof,
    gui::{self, GuiInput, GuiMessage},
    model::{Direction, FieldState, Ship, SHIP_SIZES},
    utils::{
        async_receiver::AsyncReceiver,
        log::Log,
        result::Res,
        ship_helpers::{Point, Rectangle},
        threads::select_first,
    },
};

mod board_creation;
mod game_loop;
pub mod main;

/// Possible message received from another player
#[derive(Debug, Serialize, Deserialize)]
pub enum GameMessage {
    BoardDeclaration(CorrectnessProof<BoardDeclarationCircuit>, [u8; 32]),
    AskForField(u8, u8),
    FieldProof(CorrectnessProof<FieldDeclarationCircuit>, FieldState),
}

#[derive(Debug, Clone)]
pub struct GameState {
    pub board: BoardDeclarationCircuit,
    pub their_hash: [u8; 32],
    pub our_role: Player,
    pub our_shots: Vec<(u8, u8, FieldState)>,
    pub their_shots: Vec<(u8, u8, FieldState)>,
    pub turn_of: Player,
}

const MAIN_SCREEN: &str = "

Witamy w grze w statki!
   create address:port password => create game
   join address:port password => join game
   msg name info => send msg to the second player
   put x y (left/down) => put a ship on your board (coordinates in range from 1 to 10)
   shoot x y => shoot at the position (x, y)
   Ctrl-C => Interrupt
   Ctrl-D => Exit\n
";

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

    struct Screen(Vec<Vec<char>>);

    impl Screen {
        fn new(x: usize, y: usize) -> Self {
            Screen(vec![vec![' '; x]; y])
        }
        fn draw(&mut self, r: Rectangle, c: char) {
            let (i1, i2) = r.as_interval_pair();

            // println!("{:#?}", (i1, i2));

            for x in i1.0..i1.1 {
                for y in i2.0..i2.1 {
                    self.0[y as usize][x as usize] = c;
                }
            }
        }
        fn draw_ship(&mut self, ship: Ship, offset: Point) {
            let r: Rectangle = ship.into();
            // println!("{:#?}", r);
            self.draw(r + (offset, offset), '#');
        }

        fn draw_board(&mut self, ships: Vec<Ship>, offset: Point) {
            self.draw((offset, offset + (12, 1)).into(), '-');
            self.draw((offset + (0, 11), offset + (12, 12)).into(), '-');
            self.draw((offset + (0, 1), offset + (1, 11)).into(), '|');
            self.draw((offset + (11, 1), offset + (12, 11)).into(), '|');
            self.draw((offset + (1, 1), offset + (11, 11)).into(), '~');
            for ship in ships {
                self.draw_ship(ship, offset);
            }
        }

        fn draw_shots(&mut self, shots: Vec<(u8, u8, FieldState)>, offset: Point) {
            for (x, y, state) in shots {
                self.draw(
                    (offset + (x as i8, y as i8)).into(),
                    match state {
                        FieldState::Empty => 'X',
                        FieldState::Occupied => '*',
                    },
                )
            }
        }

        fn to_string(self) -> String {
            self.0
                .into_iter()
                .map(|mut v| {
                    v.push('\n');
                    v.into_iter().collect::<String>()
                })
                .collect()
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
                        if words[0] == "shoot" {
                            return Ok(GuiInput::Shoot(
                                words[1].parse().unwrap(),
                                words[2].parse().unwrap(),
                            ));
                        }
                        if words[0] == "put" {
                            if words.len() != 4 {
                                self.log_message("Invalid command")?;
                                continue;
                            }
                            if let GuiMessage::BoardConstruction(inc_board) = &self.state {
                                return Ok(GuiInput::PutShip(Ship {
                                    x: words[1].parse().unwrap(),
                                    y: words[2].parse().unwrap(),
                                    size: SHIP_SIZES[inc_board.0.len()],
                                    direction: {
                                        if words[3] == "down" {
                                            Direction::Vertical
                                        } else {
                                            Direction::Horizontal
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
                    // let mut s = Screen::new(15, 15);
                    // s.draw(((1,1),(3,3)).into(), 'X');
                    // s.draw_board(vec![], (0,0).into());
                    // self.log_message(&s.to_string()).unwrap();
                    self.log_message(MAIN_SCREEN).unwrap();
                }
                GuiMessage::Lobby => {
                    self.log_message("\n\nLobby\n").unwrap();
                }
                GuiMessage::BoardConstruction(board) => {
                    // self.log_message(&format!("{:#?}", board)).unwrap();
                    let mut s = Screen::new(15, 15);
                    s.draw_board(board.0.clone(), (3, 3).into());
                    self.log_message(&s.to_string()).unwrap();
                }
                GuiMessage::PrintGameState(state) => {
                    let mut s = Screen::new(30, 15);
                    s.draw_board(state.board.board.ships.to_vec(), (3, 3).into());
                    s.draw_board(vec![], (18, 3).into());
                    s.draw_shots(state.our_shots.clone(), (18, 3).into());
                    s.draw_shots(state.their_shots.clone(), (3, 3).into());
                    self.log_message(&s.to_string()).unwrap();
                }
                _ => {}
            }
        }
    }

    let (reader, writer) = Readline::new("> ".to_string()).unwrap();

    let (s_input, mut r_input) = async_channel::unbounded::<GuiMessage>();
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
            match select_first(cli.receive_input(), get_input_from(&mut r_input)).await? {
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

    block_on(select_first::<(), ()>(
        f,
        run_logic_async(AsyncReceiver(r_output), s_input),
    ))
    .unwrap();
}
