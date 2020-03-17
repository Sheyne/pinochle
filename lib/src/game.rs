use self::core::*;
use either::Either;
use serde::{Deserialize, Serialize};
pub use Game::*;
pub mod core;
pub mod states;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum Game {
    Bidding(states::Bidding),
    SelectingTrump(states::SelectingTrump),
    PassingCards(states::PassingCards),
    ReturningCards(states::ReturningCards),
    Playing(states::Playing),
    FinishedRound(states::FinishedRound),
    Finished,
}

impl Game {
    pub fn new(first_player: Player, hands: PlayerMap<Vec<Card>>) -> Game {
        states::Bidding::new(first_player, states::hands_to_option(hands)).into()
    }

    pub fn play(&mut self, player: Player, input: Input) -> Result<(), String> {
        use Input::*;

        if !self.can_play(player) {
            Err("Not your turn".to_owned())?
        }

        let mut input_state = Game::Finished;
        std::mem::swap(self, &mut input_state);

        let (next, err) = match (input_state, input) {
            (Bidding(state), Bid(amount)) => next_and_error(state.bid(amount)),
            (Bidding(state), Pass) => next_and_error(state.pass()),
            (SelectingTrump(state), SelectSuit(suit)) => (state.select(suit).into(), Ok(())),
            (PassingCards(state), PassCards(cards)) => (state.pass(cards)?.into(), Ok(())),
            (ReturningCards(state), PassCards(cards)) => (state.pass(cards)?.into(), Ok(())),
            (Playing(state), Play(card)) => next_and_error(state.play(card)),
            (FinishedRound(state), Next) => (state.next().into(), Ok(())),
            (input_state, _) => (input_state, Err("".to_owned())),
        };

        *self = next;
        err
    }

    pub fn hand(&self, player: Player) -> Option<&[Option<Card>]> {
        match self {
            Bidding(b) => Some(b.hand(player)),
            SelectingTrump(b) => Some(b.hand(player)),
            PassingCards(b) => Some(b.hand(player)),
            ReturningCards(b) => Some(b.hand(player)),
            Playing(b) => Some(b.hand(player)),
            FinishedRound(b) => Some(b.hand(player)),
            Finished => None,
        }
    }

    pub fn can_play(&self, player: Player) -> bool {
        match self {
            Bidding(b) => b.turn() == player,
            SelectingTrump(b) => b.turn() == player,
            PassingCards(b) => b.turn() == player,
            ReturningCards(b) => b.turn() == player,
            Playing(b) => b.turn() == player,
            FinishedRound(_) => true,
            Finished => false,
        }
    }

    pub fn bidding(&self) -> Option<&states::Bidding> {
        match self {
            Bidding(x) => Some(x),
            _ => None,
        }
    }
    pub fn selecting_trump(&self) -> Option<&states::SelectingTrump> {
        match self {
            SelectingTrump(x) => Some(x),
            _ => None,
        }
    }
    pub fn playing(&self) -> Option<&states::Playing> {
        match self {
            Playing(x) => Some(x),
            _ => None,
        }
    }
    pub fn finished(&self) -> Option<&()> {
        match self {
            Finished => Some(&()),
            _ => None,
        }
    }
    pub fn finished_round(&self) -> Option<&states::FinishedRound> {
        match self {
            FinishedRound(x) => Some(x),
            _ => None,
        }
    }
}

impl states::Project for Game {
    fn project(&self, player: Player) -> Self {
        match self {
            Bidding(b) => b.project(player).into(),
            SelectingTrump(b) => b.project(player).into(),
            PassingCards(b) => b.project(player).into(),
            ReturningCards(b) => b.project(player).into(),
            Playing(b) => b.project(player).into(),
            FinishedRound(b) => b.project(player).into(),
            Finished => Finished,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum Input {
    Bid(usize),
    Pass,
    SelectSuit(Suit),
    PassCards(Option<[Card; 4]>),
    Play(Card),
    Next,
}

impl Input {
    pub fn mask(&self) -> Input {
        match self {
            Input::Bid(a) => Input::Bid(*a),
            Input::Pass => Input::Pass,
            Input::SelectSuit(suit) => Input::SelectSuit(*suit),
            Input::PassCards(_) => Input::PassCards(None),
            Input::Play(card) => Input::Play(*card),
            Input::Next => Input::Next,
        }
    }
}

fn next_and_error<L, R>(s: Either<(L, Option<&str>), R>) -> (Game, Result<(), String>)
where
    L: Into<Game>,
    R: Into<Game>,
{
    match s {
        Either::Left((state, err)) => (state.into(), err.map_or(Ok(()), |e| Err(e.to_owned()))),
        Either::Right(state) => (state.into(), Ok(())),
    }
}

impl<T, U> From<Either<T, U>> for Game
where
    T: Into<Game>,
    U: Into<Game>,
{
    fn from(val: Either<T, U>) -> Self {
        match val {
            Either::Left(t) => t.into(),
            Either::Right(t) => t.into(),
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

impl From<states::PassingCards> for Game {
    fn from(val: states::PassingCards) -> Self {
        PassingCards(val)
    }
}

impl From<states::ReturningCards> for Game {
    fn from(val: states::ReturningCards) -> Self {
        ReturningCards(val)
    }
}

impl From<states::Playing> for Game {
    fn from(val: states::Playing) -> Self {
        Playing(val)
    }
}

impl From<states::Finished> for Game {
    fn from(_: states::Finished) -> Self {
        Finished
    }
}

impl From<states::FinishedRound> for Game {
    fn from(val: states::FinishedRound) -> Self {
        FinishedRound(val)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const HX: Card = Card {
        suit: Suit::Heart,
        rank: Rank::Ten,
    };

    #[test]
    fn simple_round() -> Result<(), String> {
        let hands = PlayerMap::new(
            vec![HX, HX, HX, HX],
            vec![HX, HX, HX, HX],
            vec![HX, HX, HX, HX],
            vec![HX, HX, HX, HX],
        );

        let mut game = Game::new(Player::A, hands);
        game.play(Player::A, Input::Bid(250))?;
        game.play(Player::B, Input::Pass)?;
        game.play(Player::C, Input::Bid(275))?;
        game.play(Player::D, Input::Pass)?;
        game.play(Player::C, Input::SelectSuit(Suit::Heart))?;
        game.play(Player::A, Input::PassCards(Some([HX, HX, HX, HX])))?;
        game.play(Player::C, Input::PassCards(Some([HX, HX, HX, HX])))?;
        game.play(Player::C, Input::Play(HX))?;
        game.play(Player::D, Input::Play(HX))?;
        game.play(Player::A, Input::Play(HX))?;
        game.play(Player::B, Input::Play(HX))?;
        game.play(Player::C, Input::Play(HX))?;
        game.play(Player::D, Input::Play(HX))?;
        game.play(Player::A, Input::Play(HX))?;
        game.play(Player::B, Input::Play(HX))?;
        game.play(Player::C, Input::Play(HX))?;
        game.play(Player::D, Input::Play(HX))?;
        game.play(Player::A, Input::Play(HX))?;
        game.play(Player::B, Input::Play(HX))?;
        game.play(Player::C, Input::Play(HX))?;
        game.play(Player::D, Input::Play(HX))?;
        game.play(Player::A, Input::Play(HX))?;
        game.play(Player::B, Input::Play(HX))?;
        let finished_round = game.finished_round().unwrap();
        assert_eq!(
            finished_round.taken(),
            [
                vec![HX, HX, HX, HX, HX, HX, HX, HX, HX, HX, HX, HX, HX, HX, HX, HX,],
                vec![],
            ]
        );
        game.play(Player::C, Input::Next)?;
        Ok(())
    }
}
