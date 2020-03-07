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
    pub ready: PlayerMap<bool>,
    pub player: Player,
}

impl TableState {
    pub fn new(player: Player) -> TableState {
        TableState {
            ready: PlayerMap::new(),
            player: player,
        }
    }
}
