#!/bin/bash
set -e

echo building wasm target
cargo build --release --target wasm32-unknown-unknown

echo setting up deploy folder
mkdir -p ./deploy/assets
cp ./target/wasm32-unknown-unknown/release/afuera.wasm ./deploy/
cp ./wasm/index.html ./deploy/
cp -r ./assets ./deploy/
cp -r ./wasm/js ./deploy/
