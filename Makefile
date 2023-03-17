network ?= devnet  # network := devnet|mainnet|testnet
sender ?= juno12jpu0gqxtslzy3lsw3xm86euqn83mdas6mflme
wasm_filename ?= cw_lottery.wasm

build:
	./bin/build

deploy:
	./bin/deploy ./artifacts/$(wasm_filename) $(network) $(sender)

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
