use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumIter, EnumString};

#[derive(
    PartialEq, Eq, Debug, EnumString, EnumIter, Display, Clone, Copy, Deserialize, Serialize,
)]
pub enum Suit {
    Diamond,
    Club,
    Heart,
    Spade,
}

#[derive(
    PartialEq,
    PartialOrd,
    Ord,
    Eq,
    Debug,
    EnumString,
    EnumIter,
    Display,
    Clone,
    Copy,
    Deserialize,
    Serialize,
)]
pub enum Rank {
    Nine,
    Jack,
    Queen,
    King,
    Ten,
    Ace,
}

#[derive(PartialEq, Eq, Debug, Clone, Copy, Deserialize, Serialize)]
pub struct Card {
    pub suit: Suit,
    pub rank: Rank,
}

#[test]
fn test_ranking() {
    assert!(Rank::Nine < Rank::Ten);
    assert!(Rank::King < Rank::Ten);
}
