#!/bin/bash

echo "Building Solana PumpFun Sniper Bot..."
cargo build --release

if [ $? -ne 0 ]; then
    echo "Build failed!"
    exit 1
fi

echo "Build successful! Starting bot..."
cargo run --release
