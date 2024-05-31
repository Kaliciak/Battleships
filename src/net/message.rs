use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum Message<T> {
    Info { sender: String, info: String },
    Error { sender: String, info: String },
    Value(T),
}
