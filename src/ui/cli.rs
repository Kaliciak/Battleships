use std::io::Write;
use std::{cell::RefCell, sync::Arc};

use async_channel::{Receiver, Sender};
use async_std::task::block_on;
use rustyline_async::{Readline, SharedWriter};

use crate::utils::threads::select_first;
use crate::{
    model::{Direction, FieldState, Ship, SHIP_SIZES},
    utils::{
        log::Log,
        result::Res,
        ship_helpers::{Point, Rectangle},
    },
};

use super::{UiInput, UiMessage};

const MAIN_SCREEN: &str = "

Witamy w grze w statki!

Command list:
    Establishing connection:
        create address:port password => create game
        join address:port password => join game
        msg name info => send msg to the second player
    Creating board
        put x y (right/down) => put a ship on your board (coordinates in range from 1 to 10)
        clear => clear the board
    Main game:
        shoot x y => shoot at the position (x, y)
    Navigating:
        Ctrl-C => Interrupt
        Ctrl-D => Exit

";

pub fn run_cli(receiver: Receiver<UiMessage>, sender: Sender<UiInput>) {
    let (mut reader, writer) = Readline::new("> ".to_string()).unwrap();

    let _ = std::thread::Builder::new()
        .stack_size(1024 * 1024)
        .spawn(|| {
            let mut cli = Cli {
                writer,
                state: Arc::new(RefCell::new(UiMessage::MainScreen)),
            };

            let mut cli_c = cli.clone();
            let sending_task = async move {
                loop {
                    sender
                        .send(get_input(&mut reader, &mut cli_c).await?)
                        .await?;
                }
                Res::Ok(())
            };

            let receiving_task = async move {
                loop {
                    match receiver.recv().await? {
                        UiMessage::Log(m) => {
                            cli.log_message(&m)?;
                        }
                        UiMessage::Exit => {
                            return Res::Ok(());
                        }
                        s => {
                            cli.state.replace_with(|_| s);
                            cli.draw();
                        }
                    }
                }
            };
            let _ = block_on(select_first(sending_task, receiving_task));
        })
        .unwrap()
        .join();
}
async fn get_input(reader: &mut Readline, cli: &mut Cli) -> Res<UiInput> {
    loop {
        let input = reader.readline().await.unwrap();
        match input {
            rustyline_async::ReadlineEvent::Line(line) => {
                reader.add_history_entry(line.clone());
                let words = line.split(' ').collect::<Vec<&str>>();
                if words.len() == 0 {
                    cli.log_message("Invalid command")?;
                    continue;
                }
                if words[0] == "clear" {
                    return Ok(UiInput::ResetBoard);
                }
                if words.len() < 3 {
                    cli.log_message("Invalid command")?;
                    continue;
                }
                if words[0] == "create" {
                    return Ok(UiInput::HostGame {
                        addr: words[1].to_owned(),
                        passwd: words[2].to_owned(),
                    });
                }
                if words[0] == "join" {
                    return Ok(UiInput::JoinGame {
                        addr: words[1].to_owned(),
                        passwd: words[2].to_owned(),
                    });
                }
                if words[0] == "msg" {
                    return Ok(UiInput::SendMessage(
                        words[1].to_owned(),
                        words[2..].join(" ").to_owned(),
                    ));
                }
                if words[0] == "shoot" {
                    return Ok(UiInput::Shoot(
                        words[1].parse().unwrap(),
                        words[2].parse().unwrap(),
                    ));
                }
                if words[0] == "put" {
                    if words.len() != 4 {
                        cli.log_message("Invalid command")?;
                        continue;
                    }
                    if let UiMessage::BoardConstruction(inc_board) =
                        cli.state.as_ref().borrow().clone()
                    {
                        return Ok(UiInput::PutShip(Ship {
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
                cli.log_message("Invalid command")?;
            }
            rustyline_async::ReadlineEvent::Eof => {
                return Ok(UiInput::Exit);
            }
            rustyline_async::ReadlineEvent::Interrupted => return Ok(UiInput::Esc),
        }
    }
}

#[derive(Clone)]
struct Cli {
    writer: SharedWriter,
    state: Arc<RefCell<UiMessage>>,
}
impl Log for Cli {
    fn log_message(&self, msg: &str) -> Res<()> {
        self.writer.clone().write_fmt(format_args!("{msg}\n"))?;
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
    fn draw(&mut self) {
        match &*self.state.as_ref().borrow() {
            UiMessage::MainScreen => {
                self.log_message(MAIN_SCREEN).unwrap();
            }
            UiMessage::Lobby => {
                self.log_message("\n\nLobby\n").unwrap();
            }
            UiMessage::BoardConstruction(board) => {
                // self.log_message(&format!("{:#?}", board)).unwrap();
                let mut s = Screen::new(15, 15);
                s.draw_board(board.0.clone(), (3, 3).into());
                self.log_message(&s.to_string()).unwrap();
            }
            UiMessage::PrintGameState(state) => {
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
