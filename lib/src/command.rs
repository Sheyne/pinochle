use super::game::core::{Player, PlayerMap};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum Command {
    JoinTable(String),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum TableCommand {
    SetPlayer(Player),
    SetReady(bool),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TableState {
    players: PlayerMap<bool>,
}
