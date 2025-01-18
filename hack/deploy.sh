#!/bin/bash

cd $(dirname "$0")/..
cargo build --release
sudo cp target/release/rott /usr/local/bin/rott
