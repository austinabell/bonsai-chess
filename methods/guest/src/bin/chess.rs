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

use std::io::Read;

use alloy_sol_types::{sol, SolType};
use chess_engine::{Evaluate, Game, GameResult};
use risc0_zkvm::guest::env;

risc0_zkvm::guest::entry!(main);

#[repr(u8)]
enum GameState {
    Ongoing = 0,
    Win = 1,
    Loss = 2,
    Draw = 3,
}

fn make_move(board_fen: &str, player_move: String) -> (Game, GameState) {
    // TODO this is a bit inefficient to handle as fen, since we are not handling
    // two player semantics and game details. Ideally this is serialized more
    // efficiently, but this is not built into this chess engine library.
    let mut game = Game::from_fen(board_fen, None, None).unwrap();

    // Play the player's move.
    match game.board.play_move(player_move.try_into().unwrap()) {
        GameResult::Continuing(board) => game.board = board,
        GameResult::Victory(_) => return (game, GameState::Win),
        GameResult::Stalemate => return (game, GameState::Draw),
        GameResult::IllegalMove(e) => {
            panic!("Illegal move: {}", e);
        }
    };

    // Calculate and play the engine's move.
    let (m, _, _) = game.board.get_best_next_move(1);
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
    (game, state)
}

type CallParams = sol! { tuple(string, string) };

fn main() {
    // Read data sent from the application contract.
    let mut input_bytes = Vec::<u8>::new();
    env::stdin().read_to_end(&mut input_bytes).unwrap();

    // Decode parameters from the scheduled call on eth.
    let (board_state, player_move) = CallParams::decode_params(&input_bytes, true).unwrap();

    // Update the player's move and calculate the engine's move.
    let (result, state) = make_move(&board_state, player_move);
    // NOTE: timer and move count not used in fen notation. Would be ideal to just
    // not include at all, but no other serialization method implemented.
    let result_fen = result.to_fen(0, 0).unwrap();

    // Commit the journal that will be received by the application contract.
    // Encoded types should match the args expected by the application callback.
    env::commit_slice(&<sol!(tuple(string, string, uint8))>::encode(&(
        board_state,
        result_fen,
        state as u8,
    )));
}
