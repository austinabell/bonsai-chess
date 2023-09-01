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
//
// SPDX-License-Identifier: Apache-2.0

pragma solidity ^0.8.17;

import {IBonsaiRelay} from "bonsai/IBonsaiRelay.sol";
import {BonsaiCallbackReceiver} from "bonsai/BonsaiCallbackReceiver.sol";

/// @title A starter application using Bonsai through the on-chain relay.
/// @dev This contract demonstrates one pattern for offloading the computation of an expensive
//       or difficult to implement function to a RISC Zero guest running on Bonsai.
contract BonsaiChess is BonsaiCallbackReceiver {
    /// @notice board state in FEN notation.
    string public fen;

    enum GameState {
        Ongoing,
        Win,
        Lose,
        Draw
    }
    GameState public gameState;

    /// @notice Image ID of the only zkVM binary to accept callbacks from.
    bytes32 public immutable fibImageId;

    /// @notice Gas limit set on the callback from Bonsai.
    /// @dev Should be set to the maximum amount of gas your callback might reasonably consume.
    uint64 private constant BONSAI_CALLBACK_GAS_LIMIT = 100000;

    /// @notice Initialize the contract, binding it to a specified Bonsai relay and RISC Zero guest image.
    constructor(
        IBonsaiRelay bonsaiRelay,
        bytes32 _fibImageId
    ) BonsaiCallbackReceiver(bonsaiRelay) {
        fibImageId = _fibImageId;
        fen = string(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
        );
        gameState = GameState.Ongoing;
    }

    event BoardUpdated(string prevBoard, string nextBoard, string move);
    event GameOver(GameState gameState);

    /// @notice Callback function logic for processing verified journals from Bonsai.
    function updateBoard(
        string memory prevBoard,
        string memory nextBoard,
        string memory move,
        GameState state
    ) external onlyBonsaiCallback(fibImageId) {
        // Assert that the previous board state matches the current board state in the contract
        assert(
            keccak256(abi.encodePacked(prevBoard)) ==
                keccak256(abi.encodePacked(fen))
        );

        // Update the board state
        fen = nextBoard;
        gameState = state;

        emit BoardUpdated(prevBoard, nextBoard, move);
        if (gameState != GameState.Ongoing) {
            emit GameOver(gameState);
        }
    }

    /// @notice Sends a request to Bonsai generate an engine chess move in response to the move
    ///         submitted to this function. This move should be formatted in UCI notation
    ///         (e.g. "e2e4", "b1c3").
    /// @dev This function sends the request to Bonsai through the on-chain relay.
    ///      The request will trigger Bonsai to run the specified RISC Zero guest program with
    ///      the given input and asynchronously return the verified results via the callback below.
    function makeMove(string memory move) external {
        if (gameState != GameState.Ongoing) {
            revert("game is over");
        }
        bonsaiRelay.requestCallback(
            fibImageId,
            abi.encode(fen, move),
            address(this),
            this.updateBoard.selector,
            BONSAI_CALLBACK_GAS_LIMIT
        );
    }
}
