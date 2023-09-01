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
import { toColor, toDests } from "./util";
import { ethers } from "ethers";
import { abi, bytecode } from "./contracts/BonsaiChess.json";

export interface Unit {
  run: (el: HTMLElement) => Api;
}

export function run(element: Element) {
  const patch = init([classModule, attributesModule, eventListenersModule]);

  let cg: Api, vnode: VNode, contract: ethers.Contract;

  // let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

  function setupBoard(el: HTMLElement) {
    // TODO might want to handle keeping same game and loading previous session before initializing
    // new game. Not really worth for a PoC, though.
    const chess = new Chess();

    const cg = Chessground(el, {
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
          after: (orig: any, dest: any) => {
            chess.move({ from: orig, to: dest });
            const moveUCI = `${orig}${dest}`;

            console.log("calling make move with", moveUCI);
            contract.makeMove(moveUCI).catch(console.error);
            // chess.move(move.san);
            // cg.move(move.from, move.to);
            cg.set({
              turnColor: toColor(chess),
              movable: {
                color: toColor(chess),
                dests: toDests(chess),
              },
            });
            cg.playPremove();
          },
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
        // Just to add a bit of padding on the bottom, this is already styled.
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

  async function deployAndHandleEvents() {
    try {
      // Load environment variables
      const bonsaiRelayAddress = process.env.BONSAI_RELAY_ADDRESS;
      const chessId = process.env.CHESS_ID;
      console.log("deploying contract with", bonsaiRelayAddress, chessId);

      // Initialize provider with configurable RPC URL
      // TODO be able to configure the RPC url (though env likely)
      const provider = new ethers.JsonRpcProvider("http://localhost:8545");
      const wallet = new ethers.Wallet(
        "0x5de4111afa1a4b94908f83103eb1f1706367c2e68ca870fc3fb9a804cdab365a",
        provider
      );

      // Initialize contract factory to deploy the chess contract.
      const factory = new ethers.ContractFactory(abi, bytecode, wallet);

      // Deploy the contract and wait for the transaction to be finalized.
      const contractInstance = await factory.deploy(
        bonsaiRelayAddress,
        chessId
      );
      await contractInstance.waitForDeployment();
      console.log(contractInstance);

      // NOTE: The new Ethers API is strange and must be re-initialized like this.
      // Downgrading to Ethers v5 has issues with the rollup configuration.
      const contractAddr = await contractInstance.getAddress();
      contract = new ethers.Contract(contractAddr, abi, wallet);
      console.log("contract initialized", contractAddr);

      contract.on("*", (ev) => {
        console.log("ambiguous event");
        console.log(ev);
      });
      contract.on("BoardUpdated", (_, nextBoard) => {
        // TODO
        console.log("board updated", nextBoard);
      });

      // contract = new ethers.Contract(deployed.getAddress())
    } catch (e) {
      console.error(e);
    }
  }

  redraw();
  deployAndHandleEvents().catch(console.error);
}
