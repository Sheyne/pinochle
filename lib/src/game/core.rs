use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumIter, EnumString};

#[derive(
    PartialEq, Eq, Debug, EnumString, EnumIter, Clone, Copy, Deserialize, Serialize, Display,
)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum Suit {
    Diamond,
    Club,
    Heart,
    Spade,
}

impl Suit {
    pub fn to_string(&self) -> &str {
        match self {
            Suit::Diamond => "♦",
            Suit::Club => "♣",
            Suit::Heart => "♥",
            Suit::Spade => "♠",
        }
    }
}

#[derive(
    PartialEq,
    PartialOrd,
    Ord,
    Eq,
    Debug,
    EnumString,
    EnumIter,
    Clone,
    Copy,
    Display,
    Deserialize,
    Serialize,
)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum Rank {
    Nine,
    Jack,
    Queen,
    King,
    Ten,
    Ace,
}

impl Rank {
    pub fn to_string(&self) -> &str {
        match self {
            Rank::Nine => "9",
            Rank::Jack => "J",
            Rank::Queen => "Q",
            Rank::King => "K",
            Rank::Ten => "10",
            Rank::Ace => "A",
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct Card {
    pub suit: Suit,
    pub rank: Rank,
}

impl Card {
    pub fn to_string(&self) -> String {
        format!("{}{}", self.rank.to_string(), self.suit.to_string())
    }
}

#[test]
fn test_ranking() {
    assert!(Rank::Nine < Rank::Ten);
    assert!(Rank::King < Rank::Ten);
}

impl Iterator for Player {
    type Item = Player;

    fn next(&mut self) -> Option<Player> {
        let res = Some(*self);
        *self = Player::next(*self);
        res
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Team {
    Red = 0,
    Blue = 1,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Player {
    A = 0,
    B = 1,
    C = 2,
    D = 3,
}

impl Player {
    pub fn team(self) -> Team {
        match self {
            Player::A => Team::Red,
            Player::B => Team::Blue,
            Player::C => Team::Red,
            Player::D => Team::Blue,
        }
    }

    pub fn next(self) -> Player {
        match self {
            Player::A => Player::B,
            Player::B => Player::C,
            Player::C => Player::D,
            Player::D => Player::A,
        }
    }
}

#[test]
fn player_iterator() {
    let p = Player::C;

    for x in p {
        assert_eq!(x, Player::C);
        break;
    }

    let p = Player::C;
    let res: Vec<(usize, Player)> = vec![1, 2, 3].iter().zip(p).map(|(l, r)| (*l, r)).collect();

    assert_eq!(p, Player::C);

    assert_eq!(res, vec![(1, Player::C), (2, Player::D), (3, Player::A)])
}
