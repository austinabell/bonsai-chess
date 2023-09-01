.PHONY: deploy build watch serve start-relay

deploy:
	RISC0_DEV_MODE=true forge script script/Deploy.s.sol --rpc-url http://localhost:8545 --broadcast 2>&1 | tee deploy.log
	@echo "If you want to update your environment variables based on the logs, run the following commands:"
	@echo "BONSAI_RELAY_ADDRESS=$(shell grep 'Deployed BonsaiTestRelay to' deploy.log | awk '{print $$4}')" >> ./frontend/.env
	@echo "CHESS_ID=$(shell grep 'Image ID for CHESS is' deploy.log | awk '{print $$6}')" >> ./frontend/.env
	@grep 'Deployed BonsaiTestRelay to' deploy.log | awk '{ print "export BONSAI_RELAY_ADDRESS=" $$4 }'
	@grep 'Image ID for CHESS is' deploy.log | awk '{ print "export CHESS_ID=" $$6 }'
	@grep 'Deployed BonsaiChess to' deploy.log | awk '{ print "export APP_ADDRESS=" $$4 }'

build:
	forge build
	mkdir -p ./frontend/src/contracts
	cp ./out/BonsaiChess.sol/BonsaiChess.json ./frontend/src/contracts/BonsaiChess.json

watch: build
	cd ./frontend && npm run watch

serve:
	cd ./frontend && npm run serve

start-relay:
	RISC0_DEV_MODE=true cargo run --bin bonsai-ethereum-relay-cli -- run --bonsai-api-url http://localhost:8082 --relay-address "$$BONSAI_RELAY_ADDRESS"
