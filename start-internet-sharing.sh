#!/usr/bin/env bash

set -e
set -x

# Internet Sharing PoC Startup Script
# This script sets up internet sharing via mesh network

MODE=${1:-"gateway"}
PEER_IP=${2:-"127.0.0.1"}

echo "Starting Internet Sharing PoC - Mode: $MODE"

if [ "$MODE" = "gateway" ]; then
    echo "Starting Gateway Machine (10.0.0.1) for internet sharing..."
    
    # Copy internet sharing config
    cp Config-internet-sharing.toml Config.toml
    
    # Start RNS-VPN-RS
    echo "Starting RNS-VPN-RS for internet sharing..."
    RNS_VPN_PRIVKEY_PATH="./internet-privkey.pem" \
    RNS_VPN_SIGNKEY_PATH="./internet-signkey.pem" \
    RUST_LOG=info \
    sudo -E target/release/rns-vpn -p 4242 -f $PEER_IP:4243 &
    
    VPN_PID=$!
    echo "RNS-VPN-RS started with PID: $VPN_PID"
    
    # Wait for VPN to establish
    echo "Waiting for VPN interface to be ready..."
    sleep 10
    
    # Configure internet sharing
    echo "Configuring internet sharing..."
    
    # Enable IP forwarding
    echo 1 > /proc/sys/net/ipv4/ip_forward
    
    # Get internet interface
    INTERNET_IFACE=$(ip route | grep default | awk '{print $5}' | head -1)
    echo "Internet interface: $INTERNET_IFACE"
    
    # Configure NAT for VPN subnet
    iptables -t nat -A POSTROUTING -s 10.0.0.0/24 -o $INTERNET_IFACE -j MASQUERADE
    iptables -A FORWARD -i rip0 -o $INTERNET_IFACE -j ACCEPT
    iptables -A FORWARD -i $INTERNET_IFACE -o rip0 -j ACCEPT
    
    echo "Internet sharing configured!"
    echo "Gateway machine is sharing internet via mesh network"
    
    # Test connectivity
    echo "Testing meshnet connectivity..."
    ./test-mesh-connectivity.sh
    
    echo "Gateway is running!"
    echo "VPN PID: $VPN_PID"
    echo ""
    echo "Client machine can now access internet via this gateway"
    
elif [ "$MODE" = "client" ]; then
    echo "Starting Client Machine (10.0.0.2) for internet sharing..."
    
    # Create client config
    cat > Config.toml << EOF
vpn_ip = "10.0.0.2/24"

[peers]
"10.0.0.1" = "REPLACE_WITH_GATEWAY_DESTINATION_HASH"

[announce_freq_secs]
announce_freq_secs = 5
EOF
    
    # Start RNS-VPN-RS
    echo "Starting RNS-VPN-RS for internet sharing..."
    RNS_VPN_PRIVKEY_PATH="./internet-privkey.pem" \
    RNS_VPN_SIGNKEY_PATH="./internet-signkey.pem" \
    RUST_LOG=info \
    sudo -E target/release/rns-vpn -p 4243 -f $PEER_IP:4242 &
    
    VPN_PID=$!
    echo "RNS-VPN-RS started with PID: $VPN_PID"
    
    # Wait for VPN to establish
    echo "Waiting for VPN interface to be ready..."
    sleep 10
    
    # Configure client routing
    echo "Configuring client routing..."
    
    # Set gateway as default route
    ip route add default via 10.0.0.1 dev rip0
    
    # Test internet access
    echo "Testing internet access via mesh network..."
    ping -c 3 8.8.8.8
    curl -s https://google.com > /dev/null && echo "Internet access working!" || echo "Internet access failed"
    
    echo "Client is running!"
    echo "VPN PID: $VPN_PID"
    echo ""
    echo "Client machine is now using internet via mesh network"
    
else
    echo "Usage: $0 <mode> <peer_ip>"
    echo "  mode: gateway or client"
    echo "  peer_ip: IP address of the peer machine"
    exit 1
fi

# Keep script running
echo "Press Ctrl+C to stop the VPN service"
trap 'echo "Stopping VPN service..."; kill $VPN_PID 2>/dev/null; exit 0' INT
wait
