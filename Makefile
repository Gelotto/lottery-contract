build:
	./bin/build

build-unoptimized:
	RUSTFLAGS='-C link-arg=-s' cargo wasm

# deploy to local dev validator, assuming it's running
deploy:
	./bin/juno-deploy ./artifacts/cw_gelotto_ibc_lottery_smart_contract.wasm

# deploy to testnet
deploy-testnet:
	./bin/juno-deploy ./artifacts/cw_gelotto_ibc_lottery_smart_contract.wasm testnet

# deploy to mainnet
deploy-mainnet:
	./bin/juno-deploy ./artifacts/cw_gelotto_ibc_lottery_smart_contract.wasm mainnet

# instantiate last contract to be deployed
instantiate:
	./bin/juno-instantiate localnet '$(msg)'

# run all unit tests
test:
	RUST_BACKTRACE=1 cargo unit-test

# Generate the contract's JSONSchema JSON files in schemas/
schema:
	cargo schema
		
start-validator:
	./bin/juno-start-validator
