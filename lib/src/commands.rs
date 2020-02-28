use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use strum_macros::{Display, EnumIter, EnumString};
use super::core::*;
use super::board::*;

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
}