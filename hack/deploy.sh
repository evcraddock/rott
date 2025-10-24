#!/bin/bash

cd $(dirname "$0")/..
cargo build --release
sudo mkdir -p /usr/local/bin/
sudo cp target/release/rott /usr/local/bin/rott
