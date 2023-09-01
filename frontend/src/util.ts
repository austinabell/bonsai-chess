import { Color, Key } from "chessground/types";
import { ChessInstance, SQUARES } from "chess.js";

export function toDests(chess: ChessInstance): Map<Key, Key[]> {
  const dests = new Map();
  SQUARES.forEach((s) => {
    const ms = chess.moves({ square: s, verbose: true });
    if (ms.length)
      dests.set(
        s,
        ms.map((m) => m.to)
      );
  });
  return dests;
}

export function toColor(chess: ChessInstance): Color {
  return chess.turn() === "w" ? "white" : "black";
}

