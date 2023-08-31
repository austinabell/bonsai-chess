import {
  h,
  init,
  VNode,
  classModule,
  attributesModule,
  eventListenersModule,
} from "snabbdom";
import { Api } from "chessground/api";
import { Chess } from "chess.js";
import { Chessground } from "chessground";
import { aiPlay, toDests } from "./util";
// import { ethers } from "ethers";

export interface Unit {
  run: (el: HTMLElement) => Api;
}

// // Initialize provider with configurable RPC URL
// // TODO be able to configure
// const provider = new ethers.JsonRpcProvider("http://localhost:8545");
// const wallet = new ethers.Wallet(YOUR_PRIVATE_KEY, provider);

// // Deploy contract
// const factory = new ethers.ContractFactory(abi, bytecode, wallet);
// const contract = await factory.deploy();

// contract.on("BoardUpdated", (_, nextBoard) => {
//   // TODO
// });

export function run(element: Element) {
  const patch = init([classModule, attributesModule, eventListenersModule]);

  let cg: Api, vnode: VNode;

  let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

  function setupBoard(el: HTMLElement) {
    const chess = new Chess(fen);

    const cg = Chessground(el, {
      fen,
      highlight: {
        check: true,
      },
      movable: {
        color: "white",
        free: false,
        dests: toDests(chess),
      },
    });
    cg.set({
      movable: {
        events: {
          after: aiPlay(cg, chess, 1000, false),
        },
      },
    });
    return cg;
  }

  function redraw() {
    vnode = patch(vnode || element, render());
  }

  function runUnit(vnode: VNode) {
    const el = vnode.elm as HTMLElement;
    el.className = "cg-wrap";
    cg = setupBoard(el);
    window["cg"] = cg; // for messing up with it from the browser console
  }

  function render() {
    return h("div#chessground-examples", [
      h("section.blue.merida", [
        h("div.cg-wrap", {
          hook: {
            insert: runUnit,
            postpatch: runUnit,
          },
        }),
        h("p", ""),
      ]),
      h("control", [
        h(
          "button",
          {
            on: {
              click() {
                cg.toggleOrientation();
              },
            },
          },
          "Toggle orientation"
        ),
      ]),
    ]);
  }

  redraw();
}
