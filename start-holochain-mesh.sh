#!/usr/bin/env bash

set -e
set -x

# Holochain Mesh Network Startup Script
# This script starts both RNS-VPN-RS and Holochain for mesh networking

NODE_ID=${1:-"1"}
NODE1_IP=${2:-"127.0.0.1"}
NODE2_IP=${3:-"127.0.0.1"}

echo "Starting Holochain Mesh Network - Node $NODE_ID"

if [ "$NODE_ID" = "1" ]; then
    echo "Starting Node 1 (10.0.0.1)..."
    
    # Start RNS-VPN-RS in background
    echo "Starting RNS-VPN-RS..."
    RNS_VPN_PRIVKEY_PATH="./node1-privkey.pem" \
    RNS_VPN_SIGNKEY_PATH="./node1-signkey.pem" \
    RUST_LOG=info \
    sudo -E target/release/rns-vpn -p 4242 -f $NODE2_IP:4243 &
    
    VPN_PID=$!
    echo "RNS-VPN-RS started with PID: $VPN_PID"
    
    # Wait for VPN to establish
    echo "Waiting for VPN interface to be ready..."
    sleep 10
    
    # Test connectivity
    ./test-mesh-connectivity.sh
    
    # Start Holochain
    echo "Starting Holochain..."
    holochain --config holochain-node1.toml &
    
    HOLOCHAIN_PID=$!
    echo "Holochain started with PID: $HOLOCHAIN_PID"
    
    echo "Node 1 is running!"
    echo "VPN PID: $VPN_PID"
    echo "Holochain PID: $HOLOCHAIN_PID"
    
elif [ "$NODE_ID" = "2" ]; then
    echo "Starting Node 2 (10.0.0.2)..."
    
    # Start RNS-VPN-RS in background
    echo "Starting RNS-VPN-RS..."
    RNS_VPN_PRIVKEY_PATH="./node2-privkey.pem" \
    RNS_VPN_SIGNKEY_PATH="./node2-signkey.pem" \
    RUST_LOG=info \
    sudo -E target/release/rns-vpn -p 4243 -f $NODE1_IP:4242 &
    
    VPN_PID=$!
    echo "RNS-VPN-RS started with PID: $VPN_PID"
    
    # Wait for VPN to establish
    echo "Waiting for VPN interface to be ready..."
    sleep 10
    
    # Test connectivity
    ./test-mesh-connectivity.sh
    
    # Start Holochain
    echo "Starting Holochain..."
    holochain --config holochain-node2.toml &
    
    HOLOCHAIN_PID=$!
    echo "Holochain started with PID: $HOLOCHAIN_PID"
    
    echo "Node 2 is running!"
    echo "VPN PID: $VPN_PID"
    echo "Holochain PID: $HOLOCHAIN_PID"
    
else
    echo "Usage: $0 <node_id> [node1_ip] [node2_ip]"
    echo "  node_id: 1 or 2"
    echo "  node1_ip: IP address of node 1 (default: 127.0.0.1)"
    echo "  node2_ip: IP address of node 2 (default: 127.0.0.1)"
    exit 1
fi

# Keep script running
echo "Press Ctrl+C to stop all services"
trap 'echo "Stopping services..."; kill $VPN_PID $HOLOCHAIN_PID 2>/dev/null; exit 0' INT
wait
