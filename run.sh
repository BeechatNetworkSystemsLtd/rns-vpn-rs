#!/usr/bin/env bash

set -e
set -x

cargo build --release
RNS_VPN_PRIVKEY_PATH="./privkey.pem" \
RNS_VPN_SIGNKEY_PATH="./signkey.pem" \
RUST_LOG=debug RUST_BACKTRACE=1 \
  sudo -E target/release/rns-vpn -p 4242 -f 192.168.1.129:4243 #-i "vpn-test-client"

exit 0
