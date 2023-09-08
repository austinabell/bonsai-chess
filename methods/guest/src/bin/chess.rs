// Copyright 2023 RISC Zero, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![no_main]

use chess_engine::{Evaluate, Game, GameResult, Move};
use risc0_alloy::{sol, EthParams};

derisc0::entry!(main);

#[repr(u8)]
enum GameState {
    Ongoing = 0,
    Win = 1,
    Loss = 2,
    Draw = 3,
}

/// Custom formatting of moves to be minimal format of UCI. Default format is
/// verbose.
fn format_move(m: Move) -> String {
    match m {
        Move::Piece(from, to) => format!("{}{}", from, to),
        // TODO verify this format
        Move::Promotion(from, to, piece) => {
            format!("{}{} {}", from, to, piece.get_name())
        }
        Move::KingSideCastle => "O-O".to_string(),
        Move::QueenSideCastle => "O-O-O".to_string(),
        Move::Resign => "Resign".to_string(),
    }
}

fn make_move(board_fen: &str, player_move: String) -> (Game, String, GameState) {
    // TODO this is a bit inefficient to handle as fen, since we are not handling
    // two player semantics and game details. Ideally this is serialized more
    // efficiently, but this is not built into this chess engine library.
    let mut game = Game::from_fen(board_fen, None, None).unwrap();

    // Play the player's move.
    match game.board.play_move(player_move.try_into().unwrap()) {
        GameResult::Continuing(board) => game.board = board,
        GameResult::Victory(_) => return (game, "".to_string(), GameState::Win),
        GameResult::Stalemate => return (game, "".to_string(), GameState::Draw),
        GameResult::IllegalMove(e) => {
            panic!("Illegal move: {}", e);
        }
    };

    // Calculate and play the engine's move.
    let (m, _, _) = game.board.get_best_next_move(2);
    let state = match game.board.play_move(m) {
        GameResult::Continuing(board) => {
            game.board = board;
            GameState::Ongoing
        }
        GameResult::Victory(_) => GameState::Loss,
        GameResult::Stalemate => GameState::Draw,
        GameResult::IllegalMove(e) => {
            panic!("Illegal move: {}", e);
        }
    };
    (game, format_move(m), state)
}

fn main(
    eth_params: EthParams<sol!(tuple(string, string))>,
) -> EthParams<sol!(tuple(string, string, string, uint8))> {
    let EthParams((board_state, player_move)) = eth_params;
    // Update the player's move and calculate the engine's move.
    let (result, engine_move, state) = make_move(&board_state, player_move);
    // NOTE: timer and move count not used in fen notation. Would be ideal to just
    // not include at all, but no other serialization method implemented.
    let result_fen = result.to_fen(0, 0).unwrap();

    // Commit the journal that will be received by the application contract.
    // Encoded types should match the args expected by the application callback.
    EthParams((board_state, result_fen, engine_move, state as u8))
}
