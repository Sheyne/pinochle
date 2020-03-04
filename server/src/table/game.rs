use pinochle_lib::{Board, BoardState, Card, PlayerData, ResponseError};

pub struct Game {
    board: Board,
}

impl Game {
    pub fn new() -> Game {
        Game {
            board: Board::shuffle(),
        }
    }

    pub fn get(&self, index: usize) -> PlayerData {
        self.board.get(index)
    }

    pub fn play_card(&mut self, index: usize, card: &Card) -> Result<(), ResponseError> {
        if self.board.turn != index {
            return Err(ResponseError::NotYourTurn);
        }

        match self.board.play(*card) {
            Ok(BoardState::Playing) => Ok(()),
            Ok(BoardState::Finished) => {
                self.board = Board::shuffle();
                Ok(())
            }
            Err(e) => Err(ResponseError::GameError(e.to_string())),
        }
    }
}
