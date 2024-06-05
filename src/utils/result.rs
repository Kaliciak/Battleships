use std::result;

use ark_relations::r1cs::SynthesisError;
use async_std::io;

#[derive(Debug)]
pub struct Er {
    pub message: String,
}

pub type Res<T> = result::Result<T, Er>;

impl From<io::Error> for Er {
    fn from(value: io::Error) -> Self {
        Er {
            message: format!("Connection error: {}", value),
        }
    }
}

impl From<serde_json::Error> for Er {
    fn from(value: serde_json::Error) -> Self {
        Er {
            message: format!("Parsing error: {}", value),
        }
    }
}

impl From<async_channel::RecvError> for Er {
    fn from(value: async_channel::RecvError) -> Self {
        Er {
            message: format!("Channel error: {}", value),
        }
    }
}

impl<T> From<async_channel::SendError<T>> for Er {
    fn from(value: async_channel::SendError<T>) -> Self {
        Er {
            message: format!("Channel error: {}", value),
        }
    }
}

impl From<SynthesisError> for Er {
    fn from(value: SynthesisError) -> Self {
        Er {
            message: format!("Synthesis error: {}", value),
        }
    }
}
