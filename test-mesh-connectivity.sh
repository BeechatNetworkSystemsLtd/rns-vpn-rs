#!/usr/bin/env bash

set -e
set -x

echo "Testing RNS-VPN-RS Mesh Network Connectivity"

# Check if TUN interface exists
if ! ip link show | grep -q "rip"; then
    echo "ERROR: No TUN interface found. Make sure RNS-VPN-RS is running."
    exit 1
fi

# Get TUN interface name
TUN_IFACE=$(ip link show | grep "rip" | cut -d: -f2 | tr -d ' ')
echo "Found TUN interface: $TUN_IFACE"

# Check TUN interface IP
TUN_IP=$(ip addr show $TUN_IFACE | grep "inet " | awk '{print $2}' | cut -d/ -f1)
echo "TUN interface IP: $TUN_IP"

# Check if we can ping the other node
if [ "$TUN_IP" = "10.0.0.1" ]; then
    echo "Testing connectivity to Node 2 (10.0.0.2)..."
    ping -c 3 10.0.0.2
elif [ "$TUN_IP" = "10.0.0.2" ]; then
    echo "Testing connectivity to Node 1 (10.0.0.1)..."
    ping -c 3 10.0.0.1
else
    echo "WARNING: Unexpected TUN IP: $TUN_IP"
fi

# Check routing table
echo "Routing table for TUN interface:"
ip route show | grep $TUN_IFACE

# Test UDP connectivity (Holochain uses UDP)
echo "Testing UDP connectivity..."
if [ "$TUN_IP" = "10.0.0.1" ]; then
    timeout 5 nc -u 10.0.0.2 12345 < /dev/null || echo "UDP test completed (no listener expected)"
elif [ "$TUN_IP" = "10.0.0.2" ]; then
    timeout 5 nc -u 10.0.0.1 12345 < /dev/null || echo "UDP test completed (no listener expected)"
fi

echo "Mesh connectivity test completed!"
