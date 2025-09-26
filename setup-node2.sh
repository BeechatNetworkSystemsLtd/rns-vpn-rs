#!/usr/bin/env bash

set -e
set -x

echo "Setting up Node 2 (10.0.0.2) for Holochain over RNS-VPN-RS"

# Build the project
cargo build --release

# Generate keys if they don't exist
if [ ! -f "node2-privkey.pem" ] || [ ! -f "node2-signkey.pem" ]; then
    echo "Generating keys for Node 2..."
    openssl genpkey -algorithm ed25519 -out node2-signkey.pem
    openssl genpkey -algorithm X25519 -out node2-privkey.pem
fi

# Copy config
cp Config-node2.toml Config.toml

echo "Node 2 setup complete!"
echo "To start Node 2, run:"
echo "RNS_VPN_PRIVKEY_PATH=\"./node2-privkey.pem\" \\"
echo "RNS_VPN_SIGNKEY_PATH=\"./node2-signkey.pem\" \\"
echo "RUST_LOG=info \\"
echo "sudo -E target/release/rns-vpn -p 4243 -f <NODE1_IP>:4242"
echo ""
echo "Note the destination hash from the logs and update Config-node1.toml"
