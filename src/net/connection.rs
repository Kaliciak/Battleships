use std::marker::PhantomData;

use async_channel::Sender;
use async_std::{
    io::{ReadExt, WriteExt},
    net::{TcpListener, TcpStream},
};
use futures::{
    future::{select, Either},
    pin_mut, Future,
};
use serde::{Deserialize, Serialize};

use crate::utils::{
    async_receiver::AsyncReceiver,
    log::{Log, Logger},
    result::{Er, Res},
};

use super::message::Message;

pub struct Endpoint<T: Serialize + for<'a> Deserialize<'a>> {
    stream: TcpStream,
    pub second_addr: String,
    pd: PhantomData<T>,
    logger: Logger,
}

impl<T: Serialize + for<'a> Deserialize<'a>> Endpoint<T> {
    pub async fn send(&mut self, message: &Message<T>) -> Res<()> {
        let to_send = serde_json::to_vec(message).unwrap();
        let length = (to_send.len() as u32).to_be_bytes();

        self.stream.write_all(&length).await?;
        self.stream.write_all(&to_send).await?;

        Ok(())
    }

    pub async fn receive(&mut self) -> Res<Message<T>> {
        let mut length_buf = [0u8; 4];
        self.stream.read_exact(&mut length_buf).await?;
        let response_length = u32::from_be_bytes(length_buf) as usize;

        let mut buffer = vec![0; response_length];
        self.stream.read_exact(&mut buffer).await?;
        let res: Message<T> = serde_json::from_slice(&buffer)?;

        match res {
            Message::Info { sender, info } => {
                self.logger
                    .log_message(&format!("{}|  {}> {}", self.second_addr, sender, info))?;
                Ok(Message::Info { sender, info })
            }
            Message::Error { sender, info } => {
                self.logger
                    .log_message(&format!("{}|  {}!!!> {}", self.second_addr, sender, info))?;
                Err(Er { message: info })
            }
            a => Ok(a),
        }
    }

    pub async fn accept_incoming_connection(addr: &str, passwd: &str, logger: Logger) -> Res<Self> {
        logger.log_message(&format!("Listening on {}...", addr))?;
        let listener = TcpListener::bind(addr).await?;

        loop {
            let (stream, second_addr) = listener.accept().await?;
            logger.log_message(&format!("Received connection from {}", second_addr))?;
            let mut endpoint = Endpoint::<T> {
                stream,
                second_addr: second_addr.to_string(),
                pd: PhantomData,
                logger: logger.clone(),
            };

            logger.log_message("Waiting for password...")?;
            if let Message::Info { sender: _, info } = endpoint.receive().await? {
                if info == passwd {
                    logger.log_message("Correct password")?;
                    endpoint
                        .send(&Message::Info {
                            sender: "HOST".to_owned(),
                            info: "Password is correct".to_owned(),
                        })
                        .await?;
                } else {
                    logger.log_message("Incorrect password, refusing connection.")?;
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

    pub async fn create_connection_to(addr: &str, passwd: &str, logger: Logger) -> Res<Self> {
        logger.log_message(&format!("Connecting to {}...", addr))?;
        let stream = TcpStream::connect(addr).await?;
        let mut endpoint = Endpoint {
            second_addr: stream.local_addr()?.to_string().to_owned(),
            stream,
            pd: PhantomData,
            logger: logger.clone(),
        };
        endpoint
            .send(&Message::Info {
                sender: "CLIENT".to_owned(),
                info: passwd.to_owned(),
            })
            .await?;

        if let Message::Info { sender: _, info: _ } = endpoint.receive().await? {
            Ok(endpoint)
        } else {
            Err(Er {
                message: "Invalid response".to_owned(),
            })
        }
    }

    pub fn as_channel_pair(
        self,
    ) -> (
        impl Future<Output = Res<()>>,
        Sender<Message<T>>,
        AsyncReceiver<Message<T>>,
    ) {
        async fn receive_loop<T: Serialize + for<'a> Deserialize<'a>>(
            mut endpoint: Endpoint<T>,
            to_send_receiver: AsyncReceiver<Message<T>>,
            received_sender: Sender<Message<T>>,
        ) -> Res<()> {
            loop {
                let result = async {
                    let received_fut = endpoint.receive();
                    let to_send_fut = to_send_receiver.get();

                    pin_mut!(received_fut, to_send_fut);

                    match select(received_fut, to_send_fut).await {
                        Either::Left(x) => Either::Left(x.0),
                        Either::Right(x) => Either::Right(x.0),
                    }
                }
                .await;
                match result {
                    futures::future::Either::Left(x) => {
                        received_sender.send(x?).await?;
                    }
                    futures::future::Either::Right(x) => {
                        endpoint.send(&x?).await?;
                    }
                };
            }
        }

        let (s_output, r_output) = async_channel::unbounded::<Message<T>>();
        let (s_input, r_input) = async_channel::unbounded();
        (
            receive_loop(self, AsyncReceiver(r_input), s_output),
            s_input,
            AsyncReceiver(r_output),
        )
    }
}
