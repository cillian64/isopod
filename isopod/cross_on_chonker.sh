#!/bin/bash -e

# Run the cross-compile on the build server, copy back the resulting binary,
# and then run it.  Note that this assumes you are actually developing on the
# build server, i.e.  it does not sync the source between local and build
# server

REMOTE=chonker.dwt27.co.uk

ssh $REMOTE "cd isopod_electronics/isopod && ./cross_compile.sh && arm-linux-gnueabihf-strip target/armv7-unknown-linux-gnueabihf/debug/isopod"
scp $REMOTE:isopod_electronics/isopod/target/armv7-unknown-linux-gnueabihf/debug/isopod .
sudo ./isopod

