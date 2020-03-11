use super::core::*;
use either::Either;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct BiddingState;

impl Project for BiddingState {
    fn project(&self, _: Player) -> Self {
        BiddingState
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct SelectingTrumpState(Player);

impl Project for SelectingTrumpState {
    fn project(&self, _: Player) -> Self {
        let SelectingTrumpState(a) = self;
        SelectingTrumpState(*a)
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct PlayingState {
    pub play_area: Vec<Card>,
    pub taken: [Vec<Card>; NUMBER_OF_TEAMS],
    pub trump: Suit,
}

impl Project for PlayingState {
    fn project(&self, _: Player) -> Self {
        Self {
            play_area: self.play_area.clone(),
            taken: self.taken.clone(),
            trump: self.trump.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct FinishedRoundState {
    pub taken: [Vec<Card>; NUMBER_OF_TEAMS],
    pub trump: Suit,
}

impl Project for FinishedRoundState {
    fn project(&self, _: Player) -> Self {
        Self {
            taken: self.taken.clone(),
            trump: self.trump.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Finished([usize; NUMBER_OF_TEAMS]);

pub type Bidding = Active<BiddingState>;
pub type SelectingTrump = Active<SelectingTrumpState>;
pub type Playing = Active<PlayingState>;
pub type FinishedRound = Active<FinishedRoundState>;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Active<T> {
    hands: PlayerMap<Vec<Option<Card>>>,
    scores: [usize; NUMBER_OF_TEAMS],
    bids: Vec<Option<usize>>,
    turn: Player,
    initial_bidder: Player,
    state: T,
}

pub trait Project {
    fn project(&self, player: Player) -> Self;
}

impl<T> Active<T>
where
    T: Project,
{
    pub fn hand(&self, p: Player) -> &[Option<Card>] {
        self.hands.get_value(p)
    }

    fn hand_mut(&mut self, p: Player) -> &mut Vec<Option<Card>> {
        self.hands.get_value_mut(p)
    }

    pub fn score(&self, team: Team) -> usize {
        self.scores[team as usize]
    }

    fn score_mut(&mut self, team: Team) -> &mut usize {
        &mut self.scores[team as usize]
    }

    pub fn bids(&self) -> &[Option<usize>] {
        &self.bids
    }
    pub fn turn(&self) -> Player {
        self.turn
    }
    pub fn initial_bidder(&self) -> Player {
        self.initial_bidder
    }
}

impl<T> Project for Active<T>
where
    T: Project,
{
    fn project(&self, player: Player) -> Self {
        Self {
            hands: self.hands.map(|p, x| {
                if p == player {
                    x.clone()
                } else {
                    x.iter().map(|_| None).collect()
                }
            }),
            scores: self.scores.clone(),
            bids: self.bids.clone(),
            turn: self.turn.clone(),
            initial_bidder: self.initial_bidder,
            state: self.state.project(player),
        }
    }
}

impl Bidding {
    pub fn new(first_player: Player, hands: PlayerMap<Vec<Option<Card>>>) -> Bidding {
        Bidding {
            hands: hands,
            bids: Vec::new(),
            scores: [0, 0],
            turn: first_player,
            initial_bidder: first_player,
            state: BiddingState,
        }
    }

    pub fn pass(self) -> Either<(Bidding, Option<&'static str>), SelectingTrump> {
        self.do_bid(None)
    }

    pub fn bid(self, amount: usize) -> Either<(Bidding, Option<&'static str>), SelectingTrump> {
        self.do_bid(Some(amount))
    }

    fn do_bid(
        mut self,
        amount: Option<usize>,
    ) -> Either<(Bidding, Option<&'static str>), SelectingTrump> {
        if let Some(amount) = amount {
            let min_bid = 250;
            if amount < min_bid {
                return Either::Left((self, Some("Must bid be at least 250")));
            }
            if (amount - min_bid) % 25 != 0 {
                return Either::Left((self, Some("Must bid in increments of 25")));
            }
            if self.bids.iter().any(|a| a.map_or(false, |a| a >= amount)) {
                return Either::Left((self, Some("Bid must be higher than any bid")));
            }
        }

        if amount.is_none() && self.bids.len() == 0 {
            return Either::Left((self, Some("First bidder must not pass")));
        }

        self.bids.push(amount);
        self.turn = self.turn.next();

        if self.bids.len() < NUMBER_OF_PLAYERS {
            Either::Left((self, None))
        } else {
            let (_, highest_bidder) = self
                .bids
                .iter()
                .zip(self.turn)
                .max_by_key(|(bid, _)| *bid)
                .unwrap();

            Either::Right(Active {
                scores: self.scores,
                turn: highest_bidder,
                initial_bidder: self.initial_bidder,
                bids: self.bids,
                hands: self.hands,
                state: SelectingTrumpState(highest_bidder),
            })
        }
    }
}

impl SelectingTrump {
    pub fn select(self, suit: Suit) -> Playing {
        Active {
            scores: self.scores,
            turn: self.state.0,
            bids: self.bids,
            hands: self.hands,
            initial_bidder: self.initial_bidder,
            state: PlayingState {
                play_area: Vec::new(),
                taken: [Vec::new(), Vec::new()],
                trump: suit,
            },
        }
    }
}

impl Playing {
    pub fn play_area(&self) -> &[Card] {
        &self.state.play_area
    }

    pub fn trump(&self) -> Suit {
        self.state.trump
    }

    pub fn play(mut self, card: Card) -> Either<(Playing, Option<&'static str>), FinishedRound> {
        let hand = self.hand(self.turn);

        match is_legal(&self.state.play_area, hand, &card, self.state.trump) {
            Ok(_) => (),
            Err(x) => return Either::Left((self, Some(x))),
        }

        if let Some(position) = hand.iter().position(|&x| x.map_or(false, |x| x == card)) {
            self.hand_mut(self.turn).remove(position);
        } else if let Some(position) = hand.iter().position(|&x| x.is_none()) {
            self.hand_mut(self.turn).remove(position);
        } else {
            return Either::Left((self, Some("Card not in hand")));
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

            if self.hand(Player::A).len() == 0 {
                return Either::Right(self.calculate_score());
            }
        }
        return Either::Left((self, None));
    }

    fn calculate_score(mut self) -> FinishedRound {
        *self.score_mut(Team::Red) += 1;

        Active {
            hands: self.hands,
            turn: self.turn,
            initial_bidder: self.initial_bidder,
            scores: self.scores,
            bids: self.bids,
            state: FinishedRoundState {
                taken: self.state.taken,
                trump: self.state.trump,
            },
        }
    }
}

impl FinishedRound {
    pub fn trump(&self) -> Suit {
        self.state.trump
    }

    pub fn taken(&self) -> [Vec<Card>; NUMBER_OF_TEAMS] {
        self.state.taken.clone()
    }

    pub fn next(mut self) -> Either<Bidding, Finished> {
        self.bids.clear();

        if *self.scores.iter().max().unwrap() > 2000 {
            Either::Right(Finished(self.scores))
        } else {
            Either::Left(Active {
                turn: Player::A,
                initial_bidder: self.initial_bidder.next(),
                hands: hands_to_option(shuffle()),
                bids: self.bids,
                scores: self.scores,
                state: BiddingState,
            })
        }
    }
}

pub fn hands_to_option(hands: PlayerMap<Vec<Card>>) -> PlayerMap<Vec<Option<Card>>> {
    hands.map_move(|_, x| x.iter().map(|a| Some(*a)).collect())
}

fn has_suit(hand: &[Option<Card>], suit: Suit) -> bool {
    hand.iter()
        .map(|&card| card.map_or(false, |c| c.suit == suit))
        .any(|b| b)
}

pub fn is_legal(
    play_area: &[Card],
    hand: &[Option<Card>],
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
mod test {
    use super::*;

    const HA: Card = Card {
        suit: Suit::Heart,
        rank: Rank::Ace,
    };

    const HX: Card = Card {
        suit: Suit::Heart,
        rank: Rank::Ten,
    };

    #[test]
    fn simple_round() {
        let game = Bidding::new(
            Player::A,
            PlayerMap::new(
                vec![Some(HX)],
                vec![Some(HX)],
                vec![Some(HX)],
                vec![Some(HA)],
            ),
        );
        let game = game.bid(210);
        let game = game.left().unwrap().bid(210);
        let game = game.left().unwrap().bid(220);
        let game = game.left().unwrap().bid(210);
        let game = game.right().unwrap();
        assert_eq!(game.turn, Player::C);
        let game = game.select(Suit::Heart);
        assert_eq!(game.turn, Player::C);
        let (game, _) = game.play(HX).left().unwrap();
        let (game, _) = game.play(HA).left().unwrap();
        let (game, _) = game.play(HX).left().unwrap();
        let game = game.play(HX).right().unwrap();
        assert_eq!(game.state.taken, [vec![], vec![HX, HA, HX, HX]]);
        let game = game.next();
        game.left().unwrap();
    }
}
