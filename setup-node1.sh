#!/usr/bin/env bash

set -e
set -x

echo "Setting up Node 1 (10.0.0.1) for Holochain over RNS-VPN-RS"

# Build the project
cargo build --release

# Generate keys if they don't exist
if [ ! -f "node1-privkey.pem" ] || [ ! -f "node1-signkey.pem" ]; then
    echo "Generating keys for Node 1..."
    openssl genpkey -algorithm ed25519 -out node1-signkey.pem
    openssl genpkey -algorithm X25519 -out node1-privkey.pem
fi

# Copy config
cp Config-node1.toml Config.toml

echo "Node 1 setup complete!"
echo "To start Node 1, run:"
echo "RNS_VPN_PRIVKEY_PATH=\"./node1-privkey.pem\" \\"
echo "RNS_VPN_SIGNKEY_PATH=\"./node1-signkey.pem\" \\"
echo "RUST_LOG=info \\"
echo "sudo -E target/release/rns-vpn -p 4242 -f <NODE2_IP>:4243"
echo ""
echo "Note the destination hash from the logs and update Config-node2.toml"
