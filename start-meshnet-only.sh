#!/usr/bin/env bash

set -e
set -x

# Meshnet-Only PoC Startup Script
# This script starts RNS-VPN-RS for meshnet-only communication

NODE_ID=${1:-"1"}
PEER_IP=${2:-"127.0.0.1"}

echo "Starting Meshnet-Only PoC - Node $NODE_ID"

if [ "$NODE_ID" = "1" ]; then
    echo "Starting Node 1 (10.0.0.1) for meshnet-only communication..."
    
    # Copy meshnet config
    cp Config-meshnet.toml Config.toml
    
    # Start RNS-VPN-RS
    echo "Starting RNS-VPN-RS for meshnet-only..."
    RNS_VPN_PRIVKEY_PATH="./meshnet-privkey.pem" \
    RNS_VPN_SIGNKEY_PATH="./meshnet-signkey.pem" \
    RUST_LOG=info \
    sudo -E target/release/rns-vpn -p 4242 -f $PEER_IP:4243 &
    
    VPN_PID=$!
    echo "RNS-VPN-RS started with PID: $VPN_PID"
    
    # Wait for VPN to establish
    echo "Waiting for VPN interface to be ready..."
    sleep 10
    
    # Test connectivity
    echo "Testing meshnet connectivity..."
    ./test-mesh-connectivity.sh
    
    echo "Node 1 is running for meshnet-only communication!"
    echo "VPN PID: $VPN_PID"
    echo ""
    echo "To test:"
    echo "1. Disconnect internet from both machines"
    echo "2. Start Volla Messages on both machines"
    echo "3. Verify messages work over mesh network"
    
elif [ "$NODE_ID" = "2" ]; then
    echo "Starting Node 2 (10.0.0.2) for meshnet-only communication..."
    
    # Create node 2 config
    cat > Config.toml << EOF
vpn_ip = "10.0.0.2/24"

[peers]
"10.0.0.1" = "REPLACE_WITH_NODE1_DESTINATION_HASH"

[announce_freq_secs]
announce_freq_secs = 5
EOF
    
    # Start RNS-VPN-RS
    echo "Starting RNS-VPN-RS for meshnet-only..."
    RNS_VPN_PRIVKEY_PATH="./meshnet-privkey.pem" \
    RNS_VPN_SIGNKEY_PATH="./meshnet-signkey.pem" \
    RUST_LOG=info \
    sudo -E target/release/rns-vpn -p 4243 -f $PEER_IP:4242 &
    
    VPN_PID=$!
    echo "RNS-VPN-RS started with PID: $VPN_PID"
    
    # Wait for VPN to establish
    echo "Waiting for VPN interface to be ready..."
    sleep 10
    
    # Test connectivity
    echo "Testing meshnet connectivity..."
    ./test-mesh-connectivity.sh
    
    echo "Node 2 is running for meshnet-only communication!"
    echo "VPN PID: $VPN_PID"
    echo ""
    echo "To test:"
    echo "1. Disconnect internet from both machines"
    echo "2. Start Volla Messages on both machines"
    echo "3. Verify messages work over mesh network"
    
else
    echo "Usage: $0 <node_id> <peer_ip>"
    echo "  node_id: 1 or 2"
    echo "  peer_ip: IP address of the peer machine"
    exit 1
fi

# Keep script running
echo "Press Ctrl+C to stop the VPN service"
trap 'echo "Stopping VPN service..."; kill $VPN_PID 2>/dev/null; exit 0' INT
wait
