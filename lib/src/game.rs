use either::Either::*;
use serde::{Deserialize, Serialize};
use serde_json::{from_str, to_string};

pub mod core;
pub mod states;

use self::core::*;

#[derive(Deserialize, Serialize, Debug)]
pub enum Action {
    Resign,
}

#[derive(Deserialize, Serialize, Debug)]
pub enum Message<T> {
    Action(Action),
    Play(T),
}

pub enum Game {
    Bidding(states::Bidding),
    SelectingTrump(states::SelectingTrump),
    Playing(states::Playing),
    FinishedRound(states::FinishedRound),
    Finished(states::Finished),
}

pub use Game::*;

impl Game {
    pub fn new(
        first_player: Player,
        hands: [Vec<Card>; NUMBER_OF_PLAYERS],
    ) -> states::Bidding {
        states::Game::new(first_player, states::hands_to_option(hands))
    }

    pub fn get_str(self, s: &str) -> (Game, Option<Action>, Option<String>) {
        match self {
            Bidding(b) => b.get_str(s),
            SelectingTrump(b) => b.get_str(s),
            Playing(b) => b.get_str(s),
            FinishedRound(b) => b.get_str(s),
            Finished(b) => (b.into(), None, None),
        }
    }

    pub fn hand(&self, player: Player) -> &[Option<Card>] {
        match self {
            Bidding(b) => b.hand(player),
            SelectingTrump(b) => b.hand(player),
            Playing(b) => b.hand(player),
            FinishedRound(b) => b.hand(player),
            Finished(b) => b.hand(player),
        }
    }

    pub fn turn(&self) -> Player {
        match self {
            Bidding(b) => b.turn(),
            SelectingTrump(b) => b.turn(),
            Playing(b) => b.turn(),
            FinishedRound(b) => b.turn(),
            Finished(b) => b.turn(),
        }
    }

    pub fn bidding(self) -> Option<states::Bidding> {
        match self {
            Bidding(x) => Some(x),
            _ => None,
        }
    }
    pub fn selecting_trump(self) -> Option<states::SelectingTrump> {
        match self {
            SelectingTrump(x) => Some(x),
            _ => None,
        }
    }
    pub fn playing(self) -> Option<states::Playing> {
        match self {
            Playing(x) => Some(x),
            _ => None,
        }
    }
    pub fn finished(self) -> Option<states::Finished> {
        match self {
            Finished(x) => Some(x),
            _ => None,
        }
    }
    pub fn finished_round(self) -> Option<states::FinishedRound> {
        match self {
            FinishedRound(x) => Some(x),
            _ => None,
        }
    }
}

impl From<states::Bidding> for Game {
    fn from(val: states::Bidding) -> Self {
        Bidding(val)
    }
}

impl From<states::SelectingTrump> for Game {
    fn from(val: states::SelectingTrump) -> Self {
        SelectingTrump(val)
    }
}

impl From<states::Playing> for Game {
    fn from(val: states::Playing) -> Self {
        Playing(val)
    }
}

impl From<states::Finished> for Game {
    fn from(val: states::Finished) -> Self {
        Finished(val)
    }
}

impl From<states::FinishedRound> for Game {
    fn from(val: states::FinishedRound) -> Self {
        FinishedRound(val)
    }
}

trait GameState<'a>: Into<Game> {
    type Input: Deserialize<'a>;
    type Error: Serialize;

    fn get_str(self, s: &'a str) -> (Game, Option<Action>, Option<String>) {
        match from_str(s) {
            Ok(a) => {
                let (s, a, e) = self.get_message(a);
                (s, a, to_string(&e).ok())
            }
            _ => (self.into(), None, None),
        }
    }

    fn get_message(
        self,
        message: Message<Self::Input>,
    ) -> (Game, Option<Action>, Option<Self::Error>) {
        match message {
            Message::Action(a) => (self.into(), Some(a), None),
            Message::Play(i) => {
                let (state, error) = self.receive(i);
                (state, None, error)
            }
        }
    }

    fn receive(self, input: Self::Input) -> (Game, Option<Self::Error>);
}

#[derive(Deserialize, Serialize, Debug)]
pub enum BiddingInput {
    Bid(usize),
}

impl<'a> GameState<'a> for states::Bidding {
    type Input = BiddingInput;
    type Error = ();

    fn receive(self, input: Self::Input) -> (Game, Option<Self::Error>) {
        match input {
            Self::Input::Bid(amount) => match self.bid(amount) {
                Left(bidding) => (bidding.into(), None),
                Right(selecting_trump) => (selecting_trump.into(), None),
            },
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub enum SelectingTrumpInput {
    Selection(Suit),
}

impl<'a> GameState<'a> for states::SelectingTrump {
    type Input = SelectingTrumpInput;
    type Error = ();

    fn receive(self, input: Self::Input) -> (Game, Option<Self::Error>) {
        match input {
            Self::Input::Selection(suit) => (self.select(suit).into(), None),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub enum PlayingInput {
    Play(Card),
}

impl<'a> GameState<'a> for states::Playing {
    type Input = PlayingInput;
    type Error = &'a str;

    fn receive(self, input: Self::Input) -> (Game, Option<Self::Error>) {
        match input {
            // TODO, don't discard error message
            Self::Input::Play(card) => match self.play(card) {
                Left((a, e)) => (a.into(), e),
                Right(a) => (a.into(), None),
            },
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub enum FinishedRoundInput {
    Next,
}

impl<'a> GameState<'a> for states::FinishedRound {
    type Input = FinishedRoundInput;
    type Error = ();

    fn receive(self, input: Self::Input) -> (Game, Option<Self::Error>) {
        match input {
            Self::Input::Next => match self.next() {
                Left(a) => (a.into(), None),
                Right(a) => (a.into(), None),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::to_string;

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
        let game = Game::new(Player::A, [vec![HX], vec![HX], vec![HX], vec![HA]]);
        let (game, _, _) =
            game.get_str(&to_string(&Message::Play(BiddingInput::Bid(210))).unwrap());
        let (game, _, _) =
            game.get_str(&to_string(&Message::Play(BiddingInput::Bid(210))).unwrap());
        let (game, _, _) =
            game.get_str(&to_string(&Message::Play(BiddingInput::Bid(220))).unwrap());
        let (game, _, _) =
            game.get_str(&to_string(&Message::Play(BiddingInput::Bid(210))).unwrap());
        assert_eq!(game.turn(), Player::C);
        let (game, _, _) = game.get_str(
            &to_string(&Message::Play(SelectingTrumpInput::Selection(Suit::Heart))).unwrap(),
        );
        assert_eq!(game.turn(), Player::C);
        let (game, _, _) =
            game.get_str(&to_string(&Message::Play(PlayingInput::Play(HX))).unwrap());
        let (game, _, _) =
            game.get_str(&to_string(&Message::Play(PlayingInput::Play(HA))).unwrap());
        let (game, _, _) =
            game.get_str(&to_string(&Message::Play(PlayingInput::Play(HX))).unwrap());
        let (game, _, _) =
            game.get_str(&to_string(&Message::Play(PlayingInput::Play(HX))).unwrap());
        let game = game.finished_round().unwrap();
        assert_eq!(game.taken(), [vec![], vec![HX, HA, HX, HX]]);
        game.next();
    }
}
