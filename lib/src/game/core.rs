use itertools::{chain, iproduct};
use rand::seq::SliceRandom;
use rand::thread_rng;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter, EnumString};

pub const NUMBER_OF_TEAMS: usize = 2;
pub const PLAYERS_PER_TEAM: usize = 2;
pub const NUMBER_OF_PLAYERS: usize = NUMBER_OF_TEAMS * PLAYERS_PER_TEAM;

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

pub fn shuffle() -> PlayerMap<Vec<Card>> {
    let mut cards: Vec<Card> = chain(
        iproduct!(Suit::iter(), Rank::iter()),
        iproduct!(Suit::iter(), Rank::iter()),
    )
    .map(|(s, r)| Card { suit: s, rank: r })
    .collect();

    let mut rng = thread_rng();
    cards.as_mut_slice().shuffle(&mut rng);
    let cards = cards;

    let cards_each: usize = cards.len() / NUMBER_OF_PLAYERS;
    let mut iter = cards.chunks(cards_each);

    PlayerMap::new(
        iter.next().unwrap().to_vec(),
        iter.next().unwrap().to_vec(),
        iter.next().unwrap().to_vec(),
        iter.next().unwrap().to_vec(),
    )
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

#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq, Eq, EnumString, Display)]
pub enum Team {
    Red = 0,
    Blue = 1,
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq, Eq, EnumString, Display)]
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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct PlayerMap<T> {
    values: [T; NUMBER_OF_PLAYERS],
}

impl<T> PlayerMap<T> {
    pub fn new(a: T, b: T, c: T, d: T) -> PlayerMap<T> {
        PlayerMap {
            values: [a, b, c, d],
        }
    }

    pub fn get_value(&self, p: Player) -> &T {
        &self.values[p as usize]
    }

    pub fn get_value_mut(&mut self, p: Player) -> &mut T {
        &mut self.values[p as usize]
    }

    pub fn iter<'a>(&'a self) -> impl Iterator<Item = (Player, &'a T)> {
        Player::A.zip(&self.values)
    }

    pub fn iter_all<'a>(&'a self) -> impl Iterator<Item = (Player, &'a T)> {
        Player::A.zip(&self.values)
    }

    pub fn map<F, U>(&self, f: F) -> PlayerMap<U>
    where
        F: Fn(Player, &T) -> U,
    {
        let [a, b, c, d] = &self.values;

        PlayerMap::new(
            f(Player::A, a),
            f(Player::B, b),
            f(Player::C, c),
            f(Player::D, d),
        )
    }

    pub fn map_move<F, U>(self, mut f: F) -> PlayerMap<U>
    where
        F: FnMut(Player, T) -> U,
    {
        let [a, b, c, d] = self.values;

        PlayerMap::new(
            f(Player::A, a),
            f(Player::B, b),
            f(Player::C, c),
            f(Player::D, d),
        )
    }
}

impl<T> PlayerMap<Option<T>> {
    pub fn unwrap(self) -> PlayerMap<T> {
        return self.map_move(|_, x| x.unwrap());
    }
}

impl<T> PlayerMap<T>
where
    T: Eq,
{
    pub fn get_player(&self, value: &T) -> Option<Player> {
        self.values
            .iter()
            .zip(Player::A)
            .filter(|(v, _)| v == &value)
            .map(|(_, p)| p)
            .next()
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
