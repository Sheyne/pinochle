extern crate strum;
#[macro_use]
extern crate strum_macros;

use std::cmp::Ordering;
use strum::IntoEnumIterator;

#[derive(PartialEq, Eq, Debug, EnumString, EnumIter, Display, Clone, Copy)]
pub enum Suit {
    Diamond,
    Club,
    Heart,
    Spade,
}

#[derive(PartialEq, PartialOrd, Ord, Eq, Debug, EnumString, EnumIter, Display, Clone, Copy)]
pub enum Rank {
    Nine,
    Jack,
    Queen,
    King,
    Ten,
    Ace,
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub struct Card {
    pub suit: Suit,
    pub rank: Rank,
}

#[test]
fn test_ranking() {
    assert!(Rank::Nine < Rank::Ten);
    assert!(Rank::King < Rank::Ten);
}

const NUMBER_OF_TEAMS: usize = 2;
const PLAYERS_PER_TEAM: usize = 2;
const NUMBER_OF_PLAYERS: usize = NUMBER_OF_TEAMS * PLAYERS_PER_TEAM;

pub struct Board {
    pub hands: [Vec<Card>; NUMBER_OF_PLAYERS],
    pub play_area: Vec<Card>,
    pub taken: [Vec<Card>; NUMBER_OF_TEAMS],
    pub turn: usize,
    pub trump: Suit,
}

fn next_turn(player: usize) -> usize {
    (player + 1) % NUMBER_OF_PLAYERS
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

impl Board {
pub fn play(&mut self, card: Card) -> Result<(), &'static str> {
    let hand = &mut self.hands[self.turn];

    is_legal(&self.play_area, &hand, &card, self.trump)?;

    if let Some(position) = hand.iter().position(|&x| x == card) {
        hand.remove(position);
    } else {
        return Err("Card not in hand");
    }
    self.play_area.push(card);
    self.turn = next_turn(self.turn);

    if self.play_area.len() == 4 {
        let first_player = self.turn;
        let led_suit = self.play_area[0].suit;

        let winner_rel_idx = self
            .play_area
            .iter()
            .enumerate()
            .max_by(|(_, c1), (_, c2)| compare_cards(c1, c2, &led_suit, &self.trump))
            .expect("List known to have a max")
            .0;

        let winner = (winner_rel_idx + first_player) % NUMBER_OF_PLAYERS;
        let winning_team = winner % NUMBER_OF_TEAMS;

        self.taken[winning_team].extend(self.play_area.iter());
        self.play_area.clear();
    }

    Ok(())
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
    fn simple_round() {
        let mut board = Board {
            hands: [vec![S9], vec![SX], vec![SK], vec![SQ]],
            play_area: vec![],
            taken: [vec![], vec![]],
            trump: Suit::Spade,
            turn: 0,
        };

        play(&mut board, S9).unwrap();
        play(&mut board, SX).unwrap();
        play(&mut board, SK).unwrap();
        play(&mut board, SQ).unwrap();

        assert_eq!(board.taken[0].len(), 0);
        assert_eq!(board.taken[1].len(), 4);
    }
}
