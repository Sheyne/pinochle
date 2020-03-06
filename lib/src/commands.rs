use super::core::*;
use super::game::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum Action {
    PlayCard(Card),
    Reset,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Command {
    Connect(String),
    Action(Action),
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
