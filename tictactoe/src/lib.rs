#![cfg_attr(not(feature = "std"), no_std)]
use core::{borrow::BorrowMut, convert::TryInto, ops::Index};

use concordium_std::*;

type GameId = u64;

#[derive(Serial, DeserialWithState)]
#[concordium(state_parameter = "S")]
struct State<S> {
    // we save a counter here as there are
    // no easy way to get the number of games except from
    // iterating over the state.
    ctr: u64,
    // games being played
    // game id - game
    games: StateMap<GameId, Game, S>,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Copy)]
enum Player {
    Cross(AccountAddress),
    Circle(AccountAddress),
}

impl Player {
    fn to_cell(&self) -> Cell {
        Cell::Occupied(*self)
    }
}

impl From<&Player> for Cell {
    fn from(p: &Player) -> Self {
        Cell::Occupied(*p)
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Clone, Copy)]
enum GameState {
    AwaitingOpponent,
    InProgress(Player),
    Finished(Option<Player>), // None if it was a draw, otherwise it contains the winning party.
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize)]
enum Cell {
    Empty,
    Occupied(Player),
}

#[derive(Debug, Serialize, PartialEq, Eq)]
struct Board([Cell; 9]);

impl Board {
    fn new() -> Self {
        Board(vec![Cell::Empty; 9].try_into().unwrap())
    }
}

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}

/// A game of tic tac toe!
#[derive(Debug, PartialEq, Eq, Serialize)]
struct Game {
    game_state: GameState,
    board: Board,
    cross: Player,
    circle: Option<Player>,
}

impl Game {
    /// Create a new game with an initiator.
    fn new(initiator: AccountAddress) -> Self {
        Game {
            game_state: GameState::AwaitingOpponent,
            board: Board::new(),
            cross: Player::Cross(initiator),
            circle: None,
        }
    }

    fn join(&mut self, new_player: Player) -> ContractResult<()> {
        // A player can only join a game where there's a spot open!
        ensure!(self.game_state == GameState::AwaitingOpponent, CustomContractError::InvalidJoin);
        // We don't allow people to play against themself.
        ensure!(self.cross != new_player, CustomContractError::InvalidJoin);
        // Let the player join and set it in progress.
        // Game initiator (cross) starts!
        self.circle = Some(new_player);
        self.game_state = GameState::InProgress(self.cross);
        Ok(())
    }

    fn make_move(&mut self, player: &Player, the_move: PutMove) -> ContractResult<()> {
        // A player can only make a move if its their turn.
        ensure!(Self::is_it_me(self.game_state, player), CustomContractError::NotMyTurn);
        // A player can only make valid move.
        ensure!(Self::is_valid_move(&self, &the_move), CustomContractError::InvalidMove);

        // Update the board.
        // This is hideous - we can make it better.
        self.borrow_mut().board.0[the_move.0] = player.to_cell();

        // If the game is not yet finished we let the other player
        // make their move otherwise we mark the game as finished with the outcome.
        if let (true, result) = self.is_game_finished(&player, &the_move) {
            self.borrow_mut().game_state = GameState::Finished(result)
        } else {
            self.borrow_mut().game_state = match player {
                Player::Cross(_) => GameState::InProgress(self.circle.unwrap()), // should be safeish
                Player::Circle(_) => GameState::InProgress(self.cross),
            }
        }
        Ok(())
    }

    /// Check whether the proposed 'the_move' is allowed.
    /// 1. The [Cell] must not be [Cell::Occupied]
    /// 2. 'the_move' must be within the valid range [0-9]
    fn is_valid_move(the_game: &Game, the_move: &PutMove) -> bool {
        // first check that the [Cell] is not occupied.
        match the_game.board.0.get(the_move.0).unwrap() {
            // should be safeish
            Cell::Empty => return true,
            Cell::Occupied(_) => return false,
        }
    }

    /// Return whether it's the players turn or not.
    fn is_it_me(state: GameState, player: &Player) -> bool {
        match state {
            GameState::InProgress(p) => return p == *player,
            _ => return false,
        }
    }

    /// Check if the game is finished.
    /// 1. There is a winner i.e. a player which has set 3 marks connecting vertically, horizontally or diagonally.
    /// 2. There is no winner and no more possible places to put a mark.
    /// Every round we check for a winner or if it is a draw, thus we only
    /// check winning condition based on the provided move and not checking the whole
    /// board.
    /// todo: clean this mess up!
    fn is_game_finished(&self, player: &Player, the_move: &PutMove) -> (bool, Option<Player>) {
        const MAGIC_NUM: usize = 3;
        // Checks to carry out.
        let mut vertical_check = true;
        let diagonal_check = false;

        let mut winner = Some(*player);

        if let (true, winner) = self.horizontal_check(player, the_move) {
            return (true, winner);
        } else if let (true, winner) = self.vertical_check(player, the_move) {
            return (true, winner);
        } else if *self.board.0.get(4).unwrap() == Cell::Occupied(*player) 
            && self.diagonal_check(player){ // todo: ideally 'the_move' could be used for only making one check in 'diagonal_check'.
            // We only check the diagonal if the player who made a move controls the center of the board.
            // check the diagonals
            return (true, Some(*player))
        } else if self.is_draw(){
            (true, None)
        } else {
            (false, None)
        }
    }

    // horizontally winning condition
    fn horizontal_check(&self, player: &Player, the_move: &PutMove) -> (bool, Option<Player>) {
        let mut x = true;
        let row_offset = (the_move.0 / 3) * 3;
        for i in row_offset..row_offset + 3 {
            if *self.board.0.get(i).unwrap() != Cell::Occupied(*player) {
                x = false;
                break;
            }
        }
        (x, Some(*player))
    }

    fn vertical_check(&self, player: &Player, the_move: &PutMove) -> (bool, Option<Player>) {
        let mut x = true;
        let mut column_offset = the_move.0 % 3;
        for _ in 0..3 {
            if *self.board.0.get(column_offset).unwrap() != Cell::Occupied(*player) {
                x = false;
                break;
            }
            column_offset += 3;
        }
        (x, Some(*player))
    }

    /// Checks whether the player has at least two opposite corners. 
    /// If that is the case, then the player has won as it is
    /// a precondition to call this function that the player 
    /// controls the middle.
    /// todo: this is ugly
    fn diagonal_check(&self, player: &Player) -> bool {
        let ul = *self.board.0.get(0).unwrap() == Cell::Occupied(*player);
        let ur = *self.board.0.get(2).unwrap() == Cell::Occupied(*player);
        let ll = *self.board.0.get(6).unwrap() == Cell::Occupied(*player);
        let lr = *self.board.0.get(8).unwrap() == Cell::Occupied(*player);

        return ul && lr || ur && ll;
    }

    /// Checks whether every [Cell] is occupied or not. 
    /// Note. This only makes sense in conjunction with the above checks carried
    /// out before this one.
    fn is_draw(&self) -> bool {
        for c in self.board.0.iter() {
            if *c == Cell::Empty {
                return false;
            }
        }
        true
    }
}

/// The different errors the contract can produce.
#[derive(Serialize, Debug, PartialEq, Eq, Reject, SchemaType)]
enum CustomContractError {
    #[from(ParseError)]
    ParseParams,
    InvalidGameId,
    InvalidJoin,
    NotMyTurn,
    InvalidMove,
}

#[derive(Debug, PartialEq, Eq)]
struct PutMove(usize);

impl PutMove {
    /// todo: the index should be within the range [0-9]
    fn new(idx: usize) -> Self {
        PutMove(idx)
    }
}

impl Index<PutMove> for Board {
    type Output = Cell;

    fn index(&self, index: PutMove) -> &Self::Output {
        &self.0[index.0]
    }
}

type ContractResult<A> = Result<A, CustomContractError>;

impl<S: HasStateApi> State<S> {
    fn empty(state_builder: &mut StateBuilder<S>) -> Self {
        State {
            ctr: 0,
            games: state_builder.new_map(),
        }
    }

    fn create_game(&mut self, address: AccountAddress) {
        self.games.insert(self.ctr, Game::new(address));
    }

    fn join(&mut self, game_id: u64, new_player: Player) -> ContractResult<()> {
        if let Some(the_game) = &mut self.games.get_mut(&game_id) {
            the_game.join(new_player)
        } else {
            return Err(CustomContractError::InvalidGameId);
        }
    }

    fn make_move(
        &mut self,
        game_id: u64,
        player: &Player,
        the_move: PutMove,
    ) -> ContractResult<()> {
        if let Some(mut the_game) = self.games.get_mut(&game_id) {
            the_game.make_move(player, the_move)
        } else {
            return Err(CustomContractError::InvalidGameId);
        }
    }
}

#[concordium_cfg_test]
mod tests {
    use super::*;
    use test_infrastructure::*;

    const INITIATOR: AccountAddress = AccountAddress([0u8; 32]);
    const CROSS: Player = Player::Cross(AccountAddress([0u8; 32]));
    const CIRCLE: Player = Player::Circle(AccountAddress([1u8; 32]));

    /// Test initialization succeeds.
    #[concordium_test]
    fn test_game() {
        let mut game = Game::new(INITIATOR);
        for c in game.board.0.iter() {
            claim_eq!(*c, Cell::Empty);
        }
        // The game initiator can't join his own game!
        claim_eq!(game.join(CROSS), Err(CustomContractError::InvalidJoin));
        // But another player can certainly join.
        claim!(game.join(CIRCLE).is_ok());

        // Cross starts in this game of tic tac toe!
        claim_eq!(game.make_move(&CIRCLE, PutMove::new(0)), Err(CustomContractError::NotMyTurn));
        // When it's a players turn, they should be able to make a move.
        claim!(game.make_move(&CROSS, PutMove::new(0)).is_ok());
        // One is not allowed to make two consecutive moves!
        claim_eq!(game.make_move(&CROSS, PutMove::new(0)), Err(CustomContractError::NotMyTurn));
        // one is not allowed to put a mark on top of each others
        claim_eq!(game.make_move(&CIRCLE, PutMove::new(0)), Err(CustomContractError::InvalidMove));
        // The game continues...
        claim!(game.make_move(&CIRCLE, PutMove::new(1)).is_ok());
        claim!(game.make_move(&CROSS, PutMove::new(3)).is_ok());
        claim!(game.make_move(&CIRCLE, PutMove::new(4)).is_ok());

        // The initiator wins... must be cheat.
        claim!(game.make_move(&CROSS, PutMove::new(6)).is_ok());
        claim_eq!(game.game_state, GameState::Finished(Some(CROSS)));

        // Polish this part. It is not only not Circles turn, the game is also finished!
        claim_eq!(game.make_move(&CROSS, PutMove::new(8)), Err(CustomContractError::NotMyTurn));

        // Let's play a game... Horizontally that is..
        game = Game::new(INITIATOR);
        claim!(game.join(CIRCLE).is_ok());
        
        claim!(game.make_move(&CROSS, PutMove::new(0)).is_ok());
        claim!(game.make_move(&CIRCLE, PutMove::new(3)).is_ok());

        claim!(game.make_move(&CROSS, PutMove::new(1)).is_ok());
        claim!(game.make_move(&CIRCLE, PutMove::new(4)).is_ok());

        claim!(game.make_move(&CROSS, PutMove::new(2)).is_ok());
        claim_eq!(game.game_state, GameState::Finished(Some(CROSS)));

        // Let's win via the mid game'!
        game = Game::new(INITIATOR);
        claim!(game.join(CIRCLE).is_ok());
        
        claim!(game.make_move(&CROSS, PutMove::new(0)).is_ok());
        claim!(game.make_move(&CIRCLE, PutMove::new(1)).is_ok());

        claim!(game.make_move(&CROSS, PutMove::new(4)).is_ok());
        claim!(game.make_move(&CIRCLE, PutMove::new(7)).is_ok());

        claim!(game.make_move(&CROSS, PutMove::new(8)).is_ok());
        claim_eq!(game.game_state, GameState::Finished(Some(CROSS)));

        // Let's now play a dull draw
        game = Game::new(INITIATOR);
        claim!(game.join(CIRCLE).is_ok());
        claim!(game.make_move(&CROSS, PutMove::new(0)).is_ok());
        claim!(game.make_move(&CIRCLE, PutMove::new(1)).is_ok());
        claim!(game.make_move(&CROSS, PutMove::new(2)).is_ok());
        claim!(game.make_move(&CIRCLE, PutMove::new(4)).is_ok());
        claim!(game.make_move(&CROSS, PutMove::new(3)).is_ok());
        claim!(game.make_move(&CIRCLE, PutMove::new(5)).is_ok());
        claim!(game.make_move(&CROSS, PutMove::new(7)).is_ok());
        claim!(game.make_move(&CIRCLE, PutMove::new(6)).is_ok());
        claim!(game.make_move(&CROSS, PutMove::new(8)).is_ok());
        claim_eq!(game.game_state, GameState::Finished(None));
    }
}