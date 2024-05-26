use super::Bytes;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MidiError {
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MidiMessage {
    Error(MidiError),
    PlayRequest(Bytes),
    PlayResponse(bool),
    Off,
}
