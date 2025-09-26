# Answers to Team Questions about RNS-VPN-RS PoC

## Question 1: How to practically set up a working PoC with two machines?

**Answer:** Here are the exact steps beyond the examples:

### Step-by-Step Setup:

#### 1. Prepare Both Machines
```bash
# On both machines
git clone <your-repo>
cd rns-vpn-rs
cargo build --release
```

#### 2. Setup Machine A (Node 1)
```bash
# Generate keys and configure
./setup-node1.sh

# Start VPN (replace <MACHINE_B_IP> with actual IP)
./start-holochain-mesh.sh 1 <MACHINE_B_IP>

# Note the destination hash from logs
```

#### 3. Setup Machine B (Node 2)
```bash
# Generate keys and configure
./setup-node2.sh

# Start VPN (replace <MACHINE_A_IP> with actual IP)
./start-holochain-mesh.sh 2 <MACHINE_A_IP>

# Note the destination hash from logs
```

#### 4. Exchange Destination Hashes
- Copy Machine A's destination hash to `Config-node2.toml` on Machine B
- Copy Machine B's destination hash to `Config-node1.toml` on Machine A
- Restart both machines

#### 5. Verify Connectivity
```bash
# On Machine A
ping 10.0.0.2

# On Machine B
ping 10.0.0.1
```

## Question 2: How to connect Volla Messages instances?

**Answer:** Once the VPN is established, Volla Messages will automatically use the mesh network:

1. **Start Volla Messages on both machines**
2. **The apps will discover each other** through the VPN tunnel
3. **Messages will be routed** over the Reticulum mesh network

## Question 3: Network Setup Options

### Option A: Empty Router (Meshnet Only)
```
Internet Router (DISCONNECTED)
    ↓
Local WiFi Router (NO INTERNET)
    ↓
┌─────────────┐    ┌─────────────┐
│  Machine A  │    │  Machine B  │
│ 10.0.0.1    │◄──►│ 10.0.0.2    │
│ VPN + Volla │    │ VPN + Volla │
└─────────────┘    └─────────────┘
```

**Setup:**
```bash
# Use meshnet-only setup
./setup-meshnet-only.sh
./start-meshnet-only.sh 1 <MACHINE_B_IP>  # Machine A
./start-meshnet-only.sh 2 <MACHINE_A_IP>  # Machine B
```

### Option B: Normal WiFi with Internet
```
Internet Router
    ↓
WiFi Router (WITH INTERNET)
    ↓
┌─────────────┐    ┌─────────────┐
│  Machine A  │    │  Machine B  │
│ 10.0.0.1    │◄──►│ 10.0.0.2    │
│ VPN + Volla │    │ VPN + Volla │
└─────────────┘    └─────────────┘
```

**Setup:**
```bash
# Use standard setup
./setup-node1.sh  # Machine A
./setup-node2.sh  # Machine B
./start-holochain-mesh.sh 1 <MACHINE_B_IP>  # Machine A
./start-holochain-mesh.sh 2 <MACHINE_A_IP>  # Machine B
```

## Question 4: How to verify data goes over Reticulum VPN?

**Answer:** Use these verification methods:

### Method 1: Network Monitoring
```bash
# Monitor Reticulum traffic
sudo tcpdump -i any -n port 4242 or port 4243

# Monitor VPN traffic
sudo tcpdump -i rip0 -n

# Check routing table
ip route show | grep rip0
```

### Method 2: Disconnect Internet Test
1. Start VPN on both machines
2. **Disconnect internet** from both machines
3. **Volla Messages should still work** between machines
4. This proves communication is over mesh network

### Method 3: Traffic Analysis
```bash
# Check if traffic goes through TUN interface
sudo netstat -i
# Look for rip0 interface traffic

# Monitor Reticulum logs
RUST_LOG=debug ./start-holochain-mesh.sh 1 <MACHINE_B_IP>
```

### Method 4: Automated Verification
```bash
# Run verification script
./verify-mesh-traffic.sh
```

## Question 5: Internet Sharing via VPN Tunnel

**Answer:** Yes, Machine B can use internet from Machine A via the VPN tunnel.

### Setup for Internet Sharing:

#### On Machine A (Internet Gateway):
```bash
# Use internet sharing setup
./setup-internet-sharing.sh
./start-internet-sharing.sh gateway <MACHINE_B_IP>
```

#### On Machine B (Client):
```bash
# Use internet sharing setup
./start-internet-sharing.sh client <MACHINE_A_IP>
```

### Manual Configuration (if needed):
```bash
# On Machine A (Gateway)
echo 1 > /proc/sys/net/ipv4/ip_forward
INTERNET_IFACE=$(ip route | grep default | awk '{print $5}')
iptables -t nat -A POSTROUTING -s 10.0.0.0/24 -o $INTERNET_IFACE -j MASQUERADE
iptables -A FORWARD -i rip0 -o $INTERNET_IFACE -j ACCEPT
iptables -A FORWARD -i $INTERNET_IFACE -o rip0 -j ACCEPT

# On Machine B (Client)
ip route add default via 10.0.0.1 dev rip0
```

## Practical Test Scenarios

### Scenario 1: Meshnet Only (No Internet)
```bash
# Setup
1. Connect both machines to router with NO internet
2. ./setup-meshnet-only.sh
3. ./start-meshnet-only.sh 1 <MACHINE_B_IP>  # Machine A
4. ./start-meshnet-only.sh 2 <MACHINE_A_IP>  # Machine B
5. Start Volla Messages on both machines
6. Send messages between machines
7. Verify messages work without internet
```

### Scenario 2: Internet Sharing
```bash
# Setup
1. Machine A has internet, Machine B doesn't
2. ./setup-internet-sharing.sh
3. ./start-internet-sharing.sh gateway <MACHINE_B_IP>  # Machine A
4. ./start-internet-sharing.sh client <MACHINE_A_IP>  # Machine B
5. Test internet access from Machine B
6. Start Volla Messages and verify communication
```

### Scenario 3: Hybrid Communication
```bash
# Setup
1. Both machines have internet
2. ./setup-node1.sh  # Machine A
3. ./setup-node2.sh  # Machine B
4. ./start-holochain-mesh.sh 1 <MACHINE_B_IP>  # Machine A
5. ./start-holochain-mesh.sh 2 <MACHINE_A_IP>  # Machine B
6. Disconnect internet from one machine
7. Verify messages still work over mesh network
```

## Expected Results

### Successful Setup:
- ✅ TUN interface created (rip0)
- ✅ IP addresses assigned (10.0.0.1, 10.0.0.2)
- ✅ Ping works between machines
- ✅ Volla Messages can communicate
- ✅ Traffic visible on TUN interface
- ✅ Reticulum logs show connection

### Performance Expectations:
- **Latency**: 10-50ms over mesh network
- **Throughput**: 1-10 Mbps depending on mesh quality
- **Reliability**: High (mesh networking provides redundancy)

## Troubleshooting

### Common Issues:
1. **TUN interface not created**: Run with sudo
2. **No connectivity**: Check destination hashes
3. **Internet sharing not working**: Check iptables rules
4. **Volla Messages not connecting**: Check VPN status

### Debug Commands:
```bash
# Check VPN logs
RUST_LOG=debug ./start-holochain-mesh.sh 1 <MACHINE_B_IP>

# Check network interfaces
ip addr show

# Check routing table
ip route show

# Test connectivity
./test-mesh-connectivity.sh

# Verify mesh traffic
./verify-mesh-traffic.sh
```

## Summary

The RNS-VPN-RS mesh network provides:
1. **Decentralized communication** without central servers
2. **Resilient networking** with fault tolerance
3. **Privacy** through encrypted mesh communication
4. **Offline capability** for meshnet-only scenarios
5. **Internet sharing** for hybrid scenarios

All scenarios can be tested with the provided scripts and verification methods.
