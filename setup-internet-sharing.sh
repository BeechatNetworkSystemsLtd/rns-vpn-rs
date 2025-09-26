#!/usr/bin/env bash

set -e
set -x

echo "Setting up RNS-VPN-RS for Internet Sharing PoC"
echo "This setup allows one machine to share internet with another via mesh network"

# Check if running as root
if [ "$EUID" -ne 0 ]; then
    echo "Please run as root or with sudo"
    exit 1
fi

# Build the project
echo "Building RNS-VPN-RS..."
cargo build --release

# Generate keys if they don't exist
if [ ! -f "internet-privkey.pem" ] || [ ! -f "internet-signkey.pem" ]; then
    echo "Generating keys for internet sharing setup..."
    openssl genpkey -algorithm ed25519 -out internet-signkey.pem
    openssl genpkey -algorithm X25519 -out internet-privkey.pem
    chmod 600 internet-privkey.pem internet-signkey.pem
fi

# Create internet sharing configuration
cat > Config-internet-sharing.toml << EOF
vpn_ip = "10.0.0.1/24"

[peers]
"10.0.0.2" = "REPLACE_WITH_PEER_DESTINATION_HASH"

[announce_freq_secs]
announce_freq_secs = 5
EOF

echo "Internet sharing setup complete!"
echo ""
echo "Next steps:"
echo "1. Start gateway machine: ./start-internet-sharing.sh gateway <PEER_IP>"
echo "2. Start client machine: ./start-internet-sharing.sh client <GATEWAY_IP>"
echo "3. Exchange destination hashes between machines"
echo "4. Restart both machines with correct destination hashes"
echo ""
echo "Gateway machine will share internet with client machine via mesh network"
