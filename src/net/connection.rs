use std::{borrow::Borrow, marker::PhantomData};

use async_std::{
    io::{ReadExt, WriteExt},
    net::{TcpListener, TcpStream},
};
use serde::{Deserialize, Serialize};

use crate::utils::{input::InputFilter, log::Log};

use super::result::Result;
use super::{message::Message, result::Error};

pub async fn get_net_input<T: Serialize + for<'a> Deserialize<'a>>(
    filter: &mut InputFilter,
    channel: &mut Endpoint<T>,
) -> Message<T> {
    match channel.receive().await {
        Ok(x) => {
            match x.borrow() {
                Message::Info { sender, info } => filter
                    .logger
                    .log_message(&format!("{}|  {}> {}", channel.second_addr, sender, info)),
                Message::Error { sender, info } => {
                    filter.logger.log_message(&format!(
                        "{}|  {}!!!> {}",
                        channel.second_addr, sender, info
                    ));
                    return filter.interrupt().await;
                }
                _ => {}
            };
            x
        }
        Err(e) => {
            filter.logger.log_message(&e.message);
            filter.interrupt().await
        }
    }
}

pub struct Endpoint<T: Serialize + for<'a> Deserialize<'a>> {
    stream: TcpStream,
    pub second_addr: String,
    pd: PhantomData<T>,
}

impl<T: Serialize + for<'a> Deserialize<'a>> Endpoint<T> {
    pub async fn send(&mut self, message: &Message<T>) -> Result<()> {
        let to_send = serde_json::to_vec(message).unwrap();
        let length = (to_send.len() as u32).to_be_bytes();

        self.stream.write_all(&length).await?;
        self.stream.write_all(&to_send).await?;

        Ok(())
    }

    pub async fn receive(&mut self) -> Result<Message<T>> {
        let mut length_buf = [0u8; 4];
        self.stream.read_exact(&mut length_buf).await?;
        let response_length = u32::from_be_bytes(length_buf) as usize;

        let mut buffer = vec![0; response_length];
        self.stream.read_exact(&mut buffer).await?;
        let res: Message<T> = serde_json::from_slice(&buffer).unwrap();
        Ok(res)
    }

    pub async fn accept_incoming_connection(
        addr: &str,
        passwd: &str,
        mut filter: InputFilter,
    ) -> Result<Self> {
        filter
            .logger
            .log_message(&format!("Listening on {}...", addr));
        let listener = TcpListener::bind(addr).await?;

        loop {
            let (stream, second_addr) = listener.accept().await?;
            filter
                .logger
                .log_message(&format!("Received connection from {}", second_addr));
            let mut endpoint = Endpoint::<T> {
                stream,
                second_addr: second_addr.to_string(),
                pd: PhantomData,
            };

            filter.logger.log_message("Waiting for password...");
            if let Message::Info { sender: _, info } =
                get_net_input(&mut filter, &mut endpoint).await
            {
                if info == passwd {
                    filter.logger.log_message("Correct password");
                    endpoint
                        .send(&Message::Info {
                            sender: "HOST".to_owned(),
                            info: "Password is correct".to_owned(),
                        })
                        .await?;
                } else {
                    filter
                        .logger
                        .log_message("Incorrect password, refusing connection.");
                    endpoint
                        .send(&Message::Error {
                            sender: "HOST".to_owned(),
                            info: "Incorrect password".to_owned(),
                        })
                        .await?;
                    continue;
                }
            }

            return Ok(endpoint);
        }
    }

    pub async fn create_connection_to(
        addr: &str,
        passwd: &str,
        mut filter: InputFilter,
    ) -> Result<Self> {
        filter
            .logger
            .log_message(&format!("Connecting to {}...", addr));
        let stream = TcpStream::connect(addr).await?;
        let mut endpoint = Endpoint {
            second_addr: stream.local_addr()?.to_string().to_owned(),
            stream,
            pd: PhantomData,
        };
        endpoint
            .send(&Message::Info {
                sender: "CLIENT".to_owned(),
                info: passwd.to_owned(),
            })
            .await?;

        if let Message::Info { sender: _, info: _ } =
            get_net_input(&mut filter, &mut endpoint).await
        {
            Ok(endpoint)
        } else {
            Err(Error {
                message: "Invalid response".to_owned(),
            })
        }
    }
}
