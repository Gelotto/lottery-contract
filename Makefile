network ?= devnet  # network := devnet|mainnet|testnet
sender ?= juno16g2rahf5846rxzp3fwlswy08fz8ccuwk03k57y
wasm_filename ?= 
build:
	./bin/build

deploy:
	./bin/deploy ./artifacts/cw_gelotto_ibc_lottery_smart_contract.wasm $(network) $(sender)

# instantiate last contract to be deployed
instantiate:
	./bin/juno-instantiate localnet '$(msg)'

# run all unit tests
test:
	RUST_BACKTRACE=1 cargo unit-test

# Generate the contract's JSONSchema JSON files in schemas/
schema:
	cargo schema
		
validator:
	./bin/validator
