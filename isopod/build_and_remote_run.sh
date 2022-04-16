#!/bin/bash -ex

export CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER=/usr/bin/arm-linux-gnueabihf-gcc
cargo build --target=armv7-unknown-linux-gnueabihf
ssh beacon rm -f isopod
ssh beacon sudo killall isopod || true
scp target/armv7-unknown-linux-gnueabihf/debug/isopod beacon:
ssh beacon sudo ./isopod
