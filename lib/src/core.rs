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
