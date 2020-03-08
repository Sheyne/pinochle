use self::core::*;
use either::Either;
use serde::{Deserialize, Serialize};
pub use Game::*;
pub mod core;
pub mod states;

#[derive(Serialize, Deserialize, Debug)]
pub enum Game {
    Bidding(states::Bidding),
    SelectingTrump(states::SelectingTrump),
    Playing(states::Playing),
    FinishedRound(states::FinishedRound),
    Finished,
}

impl Game {
    pub fn new(first_player: Player, hands: PlayerMap<Vec<Card>>) -> Game {
        states::Bidding::new(first_player, states::hands_to_option(hands)).into()
    }

    pub fn play(&mut self, input: Input) -> Result<(), String> {
        use Input::*;

        let mut input_state = Game::Finished;
        std::mem::swap(self, &mut input_state);

        let (next, err) = match (input_state, input) {
            (Bidding(state), Bid(amount)) => (state.bid(amount).into(), Ok(())),
            (SelectingTrump(state), SelectSuit(suit)) => (state.select(suit).into(), Ok(())),
            (Playing(state), Play(card)) => match state.play(card) {
                Either::Left((state, err)) => {
                    (state.into(), err.map_or(Ok(()), |e| Err(e.to_owned())))
                }
                Either::Right(state) => (state.into(), Ok(())),
            },
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
            Playing(b) => Some(b.hand(player)),
            FinishedRound(b) => Some(b.hand(player)),
            Finished => None,
        }
    }

    pub fn turn(&self) -> Option<Player> {
        match self {
            Bidding(b) => Some(b.turn()),
            SelectingTrump(b) => Some(b.turn()),
            Playing(b) => Some(b.turn()),
            FinishedRound(b) => Some(b.turn()),
            Finished => None,
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
    pub fn finished(self) -> Option<()> {
        match self {
            Finished => Some(()),
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

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum Input {
    Bid(usize),
    SelectSuit(Suit),
    Play(Card),
    Next,
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
    const HA: Card = Card {
        suit: Suit::Heart,
        rank: Rank::Ace,
    };

    #[test]
    fn simple_round() -> Result<(), String> {
        let hands = PlayerMap::new(vec![HX], vec![HX], vec![HX], vec![HA]);

        let mut game = Game::new(Player::A, hands);
        game.play(Input::Bid(210))?;
        game.play(Input::Bid(210))?;
        game.play(Input::Bid(220))?;
        game.play(Input::Bid(210))?;
        assert_eq!(game.turn(), Some(Player::C));
        game.play(Input::SelectSuit(Suit::Heart))?;
        assert_eq!(game.turn(), Some(Player::C));
        game.play(Input::Play(HX))?;
        game.play(Input::Play(HA))?;
        game.play(Input::Play(HX))?;
        game.play(Input::Play(HX))?;
        let game = game.finished_round().unwrap();
        assert_eq!(game.taken(), [vec![], vec![HX, HA, HX, HX]]);
        game.next();
        Ok(())
    }
}
