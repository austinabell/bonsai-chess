.PHONY: deploy build watch serve start-relay test

deploy:
	$(eval LOGFILE := deploy_$(shell date +%s).log)
	RISC0_DEV_MODE=true forge script script/Deploy.s.sol --rpc-url http://localhost:8545 --broadcast

redeploy:
	RISC0_DEV_MODE=true DEPLOY_RELAY_ADDRESS="$${BONSAI_RELAY_ADDRESS:-0x5FbDB2315678afecb367f032d93F642f64180aa3}" DEPLOY_UPLOAD_IMAGES=true forge script script/Deploy.s.sol --rpc-url http://localhost:8545 --broadcast

build:
	forge build
	mkdir -p ./frontend/src/contracts
	cp ./out/BonsaiChess.sol/BonsaiChess.json ./frontend/src/contracts/BonsaiChess.json

watch: build
	cd ./frontend && npm install && npm run watch

serve:
	cd ./frontend && npm install && npm run serve

start-relay:
	RISC0_DEV_MODE=true cargo run --bin bonsai-ethereum-relay-cli -- run --relay-address "$${BONSAI_RELAY_ADDRESS:-0x5FbDB2315678afecb367f032d93F642f64180aa3}"

test:
	forge test