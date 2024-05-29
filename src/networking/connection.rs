use std::{marker::PhantomData, net::SocketAddr};

use async_std::{
    io::{ReadExt, WriteExt},
    net::{TcpListener, TcpStream},
};
use serde::{Deserialize, Serialize};

use super::result::Result;
use super::{message::Message, result::Error};

pub struct Endpoint<T: Serialize + for<'a> Deserialize<'a>> {
    stream: TcpStream,
    second_addr: SocketAddr,
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

    pub async fn accept_incoming_connection(addr: &str, passwd: &str) -> Result<Self> {
        let listener = TcpListener::bind(addr).await?;

        loop {
            let (stream, second_addr) = listener.accept().await?;
            let mut endpoint = Endpoint::<T> {
                stream,
                second_addr,
                pd: PhantomData,
            };
            if !passwd.is_empty() {
                if let Message::Info { sender: _, info } = endpoint.receive().await? {
                    if info == passwd {
                        endpoint
                            .send(&Message::Info {
                                sender: "HOST".to_owned(),
                                info: "Password is correct".to_owned(),
                            })
                            .await?;
                    } else {
                        endpoint
                            .send(&Message::Error {
                                sender: "HOST".to_owned(),
                                info: "Incorrect password".to_owned(),
                            })
                            .await?;
                        continue;
                    }
                }
            }
            return Ok(endpoint);
        }
    }

    pub async fn create_connection_to(addr: &str, passwd: &str) -> Result<Self> {
        let stream = TcpStream::connect(addr).await?;
        let mut endpoint = Endpoint {
            stream,
            second_addr: addr.parse().unwrap(),
            pd: PhantomData,
        };
        endpoint
            .send(&Message::Info {
                sender: "CLIENT".to_owned(),
                info: passwd.to_owned(),
            })
            .await?;

        let response = endpoint.receive().await?;
        if let Message::Info { sender: _, info: _ } = response {
            Ok(endpoint)
        } else {
            Err(Error {
                message: "Incorrect password".to_owned(),
            })
        }
    }
}
