#!/usr/bin/env bash

set -e
set -x

echo "Setting up RNS-VPN-RS for Meshnet-Only PoC"
echo "This setup creates a mesh network without internet access"

# Check if running as root
if [ "$EUID" -ne 0 ]; then
    echo "Please run as root or with sudo"
    exit 1
fi

# Ensure we're on the holochain branch
git checkout holochain

# Build the project
echo "Building RNS-VPN-RS..."
cargo build --release

# Generate keys if they don't exist
if [ ! -f "meshnet-privkey.pem" ] || [ ! -f "meshnet-signkey.pem" ]; then
    echo "Generating keys for meshnet setup..."
    openssl genpkey -algorithm ed25519 -out meshnet-signkey.pem
    openssl genpkey -algorithm X25519 -out meshnet-privkey.pem
    chmod 600 meshnet-privkey.pem meshnet-signkey.pem
fi

# Create meshnet configuration
cat > Config-meshnet.toml << EOF
vpn_ip = "10.0.0.1/24"

[peers]
"10.0.0.2" = "REPLACE_WITH_PEER_DESTINATION_HASH"

[announce_freq_secs]
announce_freq_secs = 5
EOF

echo "Meshnet-only setup complete!"
echo ""
echo "Next steps:"
echo "1. Start this machine: ./start-meshnet-only.sh 1 <PEER_IP>"
echo "2. Start peer machine: ./start-meshnet-only.sh 2 <THIS_MACHINE_IP>"
echo "3. Exchange destination hashes between machines"
echo "4. Restart both machines with correct destination hashes"
echo ""
echo "To verify meshnet is working:"
echo "- Disconnect internet from both machines"
echo "- Run: ping 10.0.0.2 (from this machine)"
echo "- Run: ping 10.0.0.1 (from peer machine)"
echo "- Start Volla Messages and test communication"
