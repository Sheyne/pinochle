use super::board::*;
use super::core::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum Action {
    PlayCard(Card),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Command {
    Action(Action),
    Connect(String),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Response {
    Update(PlayerData),
    Error(ResponseError),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ResponseError {
    NotConnected,
    NotYourTurn,
    NotPlaying,
    GameError(String),
}
