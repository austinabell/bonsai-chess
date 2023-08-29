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
use ethabi::Token;
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

    println!("input_bytes: {:?}", input_bytes);

    // match ethabi::decode(&[ParamType::String, ParamType::String], &input_bytes) {
    //     Ok(decoded_tokens) => {
    //         if let ethabi::Token::String(decoded_fen) = &decoded_tokens[0] {
    //             println!("Decoded FEN: {}", decoded_fen);
    //         }

    //         if let ethabi::Token::String(decoded_next_move) = &decoded_tokens[1]
    // {             panic!("Decoded Next Move: {}", decoded_next_move);
    //         }
    //     }
    //     Err(e) => panic!("Decoding error: {:?}", e),
    // }

    // Type array passed to `ethabi::decode_whole` should match the types encoded in
    // the application contract.
    let (board_state, player_move): (String, String) =
        CallParams::decode_params(&input_bytes, true)
            .unwrap_or_else(|_| panic!("input_bytes: {:?}", input_bytes));
    // let mut input = ethabi::decode_whole(&[ParamType::String, ParamType::String],
    // &input_bytes)     .unwrap_or_else(|_| panic!("input_bytes: {:?}",
    // input_bytes))     .into_iter();
    // let board_state: String = input.next().unwrap().into_string().unwrap();
    // let player_move: String = input.next().unwrap().into_string().unwrap();

    // Run the computation.
    let (result, state) = make_move(&board_state, player_move);
    // NOTE: timer and move count not used in fen notation. Would be ideal to just
    // not include at all, but no other serialization method implemented.
    let result_fen = result.to_fen(0, 0).unwrap();

    // Commit the journal that will be received by the application contract.
    // Encoded types should match the args expected by the application callback.
    // TODO update encode to use alloy
    env::commit_slice(&ethabi::encode(&[
        Token::String(board_state),
        Token::String(result_fen),
        Token::Uint((state as u8).into()),
    ]));
}
