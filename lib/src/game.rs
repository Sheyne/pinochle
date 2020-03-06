use super::core::*;
use itertools::{chain, iproduct};
use rand::seq::SliceRandom;
use rand::thread_rng;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use strum::IntoEnumIterator;

const NUMBER_OF_TEAMS: usize = 2;
const PLAYERS_PER_TEAM: usize = 2;
const NUMBER_OF_PLAYERS: usize = NUMBER_OF_TEAMS * PLAYERS_PER_TEAM;

#[derive(Serialize, Deserialize, Debug)]
pub struct PlayerData {
    pub player: usize,
    pub hand: Vec<Card>,
    pub play_area: Vec<Card>,
    pub weve_taken: usize,
    pub theyve_taken: usize,
    pub turn: usize,
    pub trump: Suit,
}

pub struct Bidding;
pub struct SelectingTrump(Player);
pub struct Playing {
    pub play_area: Vec<Card>,
    pub taken: [Vec<Card>; NUMBER_OF_TEAMS],
    pub trump: Suit,
}
pub struct FinishedRound {
    pub taken: [Vec<Card>; NUMBER_OF_TEAMS],
    pub trump: Suit,
}
pub struct Finished;

pub enum GameEnum {
    Bidding(Game<Bidding>),
    SelectingTrump(Game<SelectingTrump>),
    Playing(Game<Playing>),
    FinishedRound(Game<FinishedRound>),
    Finished(Game<Finished>),
}

pub struct Game<T> {
    pub hands: [Vec<Card>; NUMBER_OF_PLAYERS],
    pub scores: [usize; NUMBER_OF_TEAMS],
    pub bids: Vec<usize>,
    pub turn: Player,
    pub initial_bidder: Player,
    pub state: T,
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

impl Iterator for Player {
    type Item = Player;

    fn next(&mut self) -> Option<Player> {
        let res = Some(*self);
        *self = Player::next(*self);
        res
    }
}

pub fn shuffle() -> [Vec<Card>; NUMBER_OF_PLAYERS] {
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

    [
        iter.next().unwrap().to_vec(),
        iter.next().unwrap().to_vec(),
        iter.next().unwrap().to_vec(),
        iter.next().unwrap().to_vec(),
    ]
}

impl GameEnum {
    pub fn new(first_player: Player, hands: [Vec<Card>; NUMBER_OF_PLAYERS]) -> Game<Bidding> {
        Game {
            hands: hands,
            bids: Vec::new(),
            scores: [0, 0],
            turn: first_player,
            initial_bidder: first_player,
            state: Bidding,
        }
    }

    pub fn bidding(self) -> Option<Game<Bidding>> {
        match self {
            GameEnum::Bidding(x) => Some(x),
            _ => None,
        }
    }
    pub fn selecting_trump(self) -> Option<Game<SelectingTrump>> {
        match self {
            GameEnum::SelectingTrump(x) => Some(x),
            _ => None,
        }
    }
    pub fn playing(self) -> Option<Game<Playing>> {
        match self {
            GameEnum::Playing(x) => Some(x),
            _ => None,
        }
    }
    pub fn finished(self) -> Option<Game<Finished>> {
        match self {
            GameEnum::Finished(x) => Some(x),
            _ => None,
        }
    }
    pub fn finished_round(self) -> Option<Game<FinishedRound>> {
        match self {
            GameEnum::FinishedRound(x) => Some(x),
            _ => None,
        }
    }
}

impl Game<Bidding> {
    pub fn bid(mut self, amount: usize) -> GameEnum {
        self.bids.push(amount);
        self.turn = self.turn.next();

        if self.bids.len() < NUMBER_OF_PLAYERS {
            GameEnum::Bidding(self)
        } else {
            let x: Vec<(&usize, Player)> = self.bids.iter().zip(self.turn).collect();
            println!("{:?}", x);

            let (_, highest_bidder) = self
                .bids
                .iter()
                .zip(self.turn)
                .max_by_key(|(bid, _)| *bid)
                .unwrap();

            GameEnum::SelectingTrump(Game {
                scores: self.scores,
                turn: highest_bidder,
                initial_bidder: self.initial_bidder,
                bids: self.bids,
                hands: self.hands,
                state: SelectingTrump(highest_bidder),
            })
        }
    }
}

impl From<Game<Bidding>> for GameEnum {
    fn from(val: Game<Bidding>) -> Self {
        GameEnum::Bidding(val)
    }
}

impl From<Game<SelectingTrump>> for GameEnum {
    fn from(val: Game<SelectingTrump>) -> Self {
        GameEnum::SelectingTrump(val)
    }
}

impl From<Game<Playing>> for GameEnum {
    fn from(val: Game<Playing>) -> Self {
        GameEnum::Playing(val)
    }
}

impl From<Game<Finished>> for GameEnum {
    fn from(val: Game<Finished>) -> Self {
        GameEnum::Finished(val)
    }
}

impl From<Game<FinishedRound>> for GameEnum {
    fn from(val: Game<FinishedRound>) -> Self {
        GameEnum::FinishedRound(val)
    }
}

impl Game<SelectingTrump> {
    pub fn select(self, suit: Suit) -> Game<Playing> {
        Game {
            scores: self.scores,
            turn: self.state.0,
            bids: self.bids,
            hands: self.hands,
            initial_bidder: self.initial_bidder,
            state: Playing {
                play_area: Vec::new(),
                taken: [Vec::new(), Vec::new()],
                trump: suit,
            },
        }
    }
}

impl Game<Playing> {
    // pub fn get(&self, player: usize) -> PlayerData {
    //     PlayerData {
    //         player: player,
    //         hand: self.hands[player].clone(),
    //         play_area: self.play_area.clone(),
    //         weve_taken: self.taken[0].len(),
    //         theyve_taken: self.taken[1].len(),
    //         turn: self.turn,
    //         trump: self.trump,
    //     }
    // }

    pub fn play(mut self, card: Card) -> Result<GameEnum, &'static str> {
        let hand = &mut self.hands[self.turn as usize];

        is_legal(&self.state.play_area, &hand, &card, self.state.trump)?;

        if let Some(position) = hand.iter().position(|&x| x == card) {
            hand.remove(position);
        } else {
            return Err("Card not in hand");
        }
        self.state.play_area.push(card);
        self.turn = self.turn.next();

        if self.state.play_area.len() == NUMBER_OF_PLAYERS {
            let first_player = self.turn;
            let led_suit = self.state.play_area[0].suit;

            let mut iter = self.state.play_area.iter().zip(first_player);

            // regular rust max_by returns the last winner when
            // deciding ties. This returns the first
            let (mut card, mut winner) = iter.next().unwrap();
            for (c, p) in iter {
                if compare_cards(c, card, &led_suit, &self.state.trump) == Ordering::Greater {
                    card = c;
                    winner = p;
                }
            }

            self.state.taken[winner.team() as usize].extend(self.state.play_area.iter());
            self.state.play_area.clear();

            self.turn = winner;

            if self.hands[0].len() == 0 {
                return Ok(GameEnum::FinishedRound(self.score()));
            }
        }
        return Ok(GameEnum::Playing(self));
    }

    fn score(self) -> Game<FinishedRound> {
        Game {
            hands: self.hands,
            turn: self.turn,
            initial_bidder: self.initial_bidder,
            scores: self.scores,
            bids: self.bids,
            state: FinishedRound {
                taken: self.state.taken,
                trump: self.state.trump,
            },
        }
    }
}

impl Game<FinishedRound> {
    pub fn next(mut self) -> GameEnum {
        self.bids.clear();

        if *self.scores.iter().max().unwrap() > 2000 {
            GameEnum::Finished(Game {
                turn: Player::A,
                initial_bidder: self.initial_bidder.next(),
                hands: shuffle(),
                bids: self.bids,
                scores: self.scores,
                state: Finished,
            })
        } else {
            GameEnum::Bidding(Game {
                turn: Player::A,
                initial_bidder: self.initial_bidder.next(),
                hands: shuffle(),
                bids: self.bids,
                scores: self.scores,
                state: Bidding,
            })
        }
    }
}

fn has_suit(hand: &[Card], suit: Suit) -> bool {
    hand.iter().map(|&card| card.suit).any(|s| s == suit)
}

pub fn is_legal(
    play_area: &[Card],
    hand: &[Card],
    card: &Card,
    trump: Suit,
) -> Result<(), &'static str> {
    if play_area.len() == 0 {
        Result::Ok(())
    } else {
        let intial_suit = play_area[0].suit;

        if has_suit(hand, intial_suit) {
            if intial_suit == card.suit {
                Result::Ok(())
            } else {
                Result::Err("Must follow suit")
            }
        } else {
            if has_suit(hand, trump) {
                if trump == card.suit {
                    Result::Ok(())
                } else {
                    Result::Err("Must trump")
                }
            } else {
                Result::Ok(())
            }
        }
    }
}

fn compare_cards(c1: &Card, c2: &Card, led_suit: &Suit, trump_suit: &Suit) -> Ordering {
    let s1 = c1.suit;
    let s2 = c2.suit;

    if s1 == s2 {
        c1.rank.cmp(&c2.rank)
    } else {
        if s1 == *trump_suit {
            Ordering::Greater
        } else if s2 == *trump_suit {
            Ordering::Less
        } else if s1 == *led_suit {
            Ordering::Greater
        } else if s2 == *led_suit {
            Ordering::Less
        } else {
            // order is irrelevant if neither card is trump or led suit,
            // but I don't want to make the ordering partial, so lets
            // let them compare equal

            Ordering::Equal
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const S9: Card = Card {
        suit: Suit::Spade,
        rank: Rank::Nine,
    };
    const SJ: Card = Card {
        suit: Suit::Spade,
        rank: Rank::Jack,
    };
    const SQ: Card = Card {
        suit: Suit::Spade,
        rank: Rank::Queen,
    };
    const SK: Card = Card {
        suit: Suit::Spade,
        rank: Rank::King,
    };
    const SX: Card = Card {
        suit: Suit::Spade,
        rank: Rank::Ten,
    };
    const SA: Card = Card {
        suit: Suit::Spade,
        rank: Rank::Ace,
    };

    const D9: Card = Card {
        suit: Suit::Diamond,
        rank: Rank::Nine,
    };
    const DJ: Card = Card {
        suit: Suit::Diamond,
        rank: Rank::Jack,
    };
    const DQ: Card = Card {
        suit: Suit::Diamond,
        rank: Rank::Queen,
    };
    const DK: Card = Card {
        suit: Suit::Diamond,
        rank: Rank::King,
    };
    const DX: Card = Card {
        suit: Suit::Diamond,
        rank: Rank::Ten,
    };
    const DA: Card = Card {
        suit: Suit::Diamond,
        rank: Rank::Ace,
    };

    const C9: Card = Card {
        suit: Suit::Club,
        rank: Rank::Nine,
    };
    const CJ: Card = Card {
        suit: Suit::Club,
        rank: Rank::Jack,
    };
    const CQ: Card = Card {
        suit: Suit::Club,
        rank: Rank::Queen,
    };
    const CK: Card = Card {
        suit: Suit::Club,
        rank: Rank::King,
    };
    const CX: Card = Card {
        suit: Suit::Club,
        rank: Rank::Ten,
    };
    const CA: Card = Card {
        suit: Suit::Club,
        rank: Rank::Ace,
    };

    const H9: Card = Card {
        suit: Suit::Heart,
        rank: Rank::Nine,
    };
    const HJ: Card = Card {
        suit: Suit::Heart,
        rank: Rank::Jack,
    };
    const HQ: Card = Card {
        suit: Suit::Heart,
        rank: Rank::Queen,
    };
    const HK: Card = Card {
        suit: Suit::Heart,
        rank: Rank::King,
    };
    const HX: Card = Card {
        suit: Suit::Heart,
        rank: Rank::Ten,
    };
    const HA: Card = Card {
        suit: Suit::Heart,
        rank: Rank::Ace,
    };

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

    #[test]
    fn simple_round() {
        let game = GameEnum::new(Player::A, [vec![HX], vec![HX], vec![HX], vec![HA]]);
        let game = game.bid(210);
        let game = game.bidding().unwrap().bid(210);
        let game = game.bidding().unwrap().bid(220);
        let game = game.bidding().unwrap().bid(210);
        let game = game.selecting_trump().unwrap();
        assert_eq!(game.turn, Player::C);
        let game = game.select(Suit::Heart);
        assert_eq!(game.turn, Player::C);
        let game = game.play(HX).unwrap();
        let game = game.playing().unwrap().play(HA).unwrap();
        let game = game.playing().unwrap().play(HX).unwrap();
        let game = game.playing().unwrap().play(HX).unwrap();
        let game = game.finished_round().unwrap();
        assert_eq!(game.state.taken, [vec![], vec![HX, HA, HX, HX]]);
        let game = game.next();
        game.bidding().unwrap();

        // let mut board = Board {
        //     hands: [vec![S9], vec![SX], vec![SK], vec![SQ]],
        //     play_area: vec![],
        //     taken: [vec![], vec![]],
        //     trump: Suit::Spade,
        //     turn: 0,
        // };

        // play(&mut board, S9).unwrap();
        // play(&mut board, SX).unwrap();
        // play(&mut board, SK).unwrap();
        // play(&mut board, SQ).unwrap();

        // assert_eq!(board.taken[0].len(), 0);
        // assert_eq!(board.taken[1].len(), 4);
    }
}
