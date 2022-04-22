#!/bin/bash -ex

# Do the cross-compile
export CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER=/usr/bin/arm-linux-gnueabihf-gcc
cargo build --target=armv7-unknown-linux-gnueabihf

# Clear up on the remote
ssh beacon sudo killall isopod || true

# Copy over the new binary and latest settings
rsync -P target/armv7-unknown-linux-gnueabihf/debug/isopod settings.toml beacon:

# Run the binary
ssh beacon sudo ./isopod
