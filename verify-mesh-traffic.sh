#!/usr/bin/env bash

set -e
set -x

echo "Verifying RNS-VPN-RS Mesh Network Traffic"
echo "This script helps verify that traffic is going over Reticulum mesh network"

# Check if running as root
if [ "$EUID" -ne 0 ]; then
    echo "Please run as root or with sudo"
    exit 1
fi

echo "=== Network Interface Status ==="
echo "TUN Interface:"
ip addr show | grep -A 5 "rip" || echo "No TUN interface found"

echo ""
echo "=== Routing Table ==="
echo "VPN Routes:"
ip route show | grep rip || echo "No VPN routes found"

echo ""
echo "=== Connectivity Test ==="
echo "Testing connectivity to peer..."
if ping -c 3 10.0.0.2 > /dev/null 2>&1; then
    echo "✅ Can reach peer at 10.0.0.2"
elif ping -c 3 10.0.0.1 > /dev/null 2>&1; then
    echo "✅ Can reach peer at 10.0.0.1"
else
    echo "❌ Cannot reach peer via VPN"
fi

echo ""
echo "=== Traffic Monitoring ==="
echo "Starting traffic monitoring for 30 seconds..."
echo "Look for traffic on TUN interface (rip0) and Reticulum ports (4242/4243)"
echo ""

# Monitor traffic for 30 seconds
timeout 30 bash -c '
echo "Monitoring TUN interface traffic..."
tcpdump -i rip0 -n -c 10 2>/dev/null || echo "No traffic on TUN interface"

echo ""
echo "Monitoring Reticulum traffic..."
tcpdump -i any -n port 4242 or port 4243 -c 10 2>/dev/null || echo "No Reticulum traffic"
' || true

echo ""
echo "=== Verification Commands ==="
echo "To manually verify mesh traffic:"
echo ""
echo "1. Monitor TUN interface:"
echo "   sudo tcpdump -i rip0 -n"
echo ""
echo "2. Monitor Reticulum traffic:"
echo "   sudo tcpdump -i any -n port 4242 or port 4243"
echo ""
echo "3. Check VPN logs:"
echo "   RUST_LOG=debug ./start-holochain-mesh.sh 1 <PEER_IP>"
echo ""
echo "4. Test without internet:"
echo "   - Disconnect internet from both machines"
echo "   - Verify Volla Messages still works"
echo "   - This proves communication is over mesh network"
echo ""
echo "5. Check routing:"
echo "   ip route show | grep rip0"
echo "   ip addr show rip0"
echo ""
echo "=== Expected Results ==="
echo "✅ TUN interface (rip0) should exist"
echo "✅ IP address 10.0.0.1 or 10.0.0.2 should be assigned"
echo "✅ Can ping peer machine"
echo "✅ Traffic visible on TUN interface"
echo "✅ Reticulum traffic on ports 4242/4243"
echo "✅ Volla Messages works without internet"
