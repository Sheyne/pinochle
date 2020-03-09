use super::game::{
    core::{Player, PlayerMap},
    Input,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum PlayingResponse {
    Played(Player, Input),
    Resigned(Player),
    Error(String),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum PlayingInput {
    Play(Input),
    Resign,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Command {
    JoinTable(String),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum TableCommand {
    SetPlayer(Player),
    SetReady(bool),
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct TableState {
    pub ready: PlayerMap<bool>,
    pub player: Player,
}

impl TableState {
    pub fn new(player: Player) -> TableState {
        TableState {
            ready: PlayerMap::new(false, false, false, false),
            player: player,
        }
    }
}
