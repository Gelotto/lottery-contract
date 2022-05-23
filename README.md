# CosmWasm Smart Contract

## Setting Up

Make sure `rustup` is installed and everything is up-to-date.
Make sure `wasm32-unknown-unknown` build target is installed by running:

```sh
rustup target list --installed
```

If it's not listed, do:

```sh
rustup target add wasm32-unknown-unknown
```

## Building

To produce a wasm build in `./target/wasm32-unknown-unknown/realease` do:

```
make build
```

This runs `cargo wasm`. If this gives you `error: no such subcommand`, try installing `cargo-wasm` and trying again.

```sh
cargo install --force cargo-wasm
```
