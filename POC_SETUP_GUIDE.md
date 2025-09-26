# RNS-VPN-RS Proof of Concept Setup Guide

This guide addresses the specific questions about setting up a working PoC for the RNS-VPN-RS mesh network.

## Questions & Answers

### Q1: How to practically set up a working PoC with two machines?

**Answer:** Here are the exact steps to get two machines talking over the VPN:

#### Step 1: Prepare Both Machines
```bash
# On both machines, clone and build
git clone <your-repo>
cd rns-vpn-rs
cargo build --release
```

#### Step 2: Generate Keys and Configure
```bash
# On Machine A (Node 1)
./setup-node1.sh
# Note the destination hash from logs

# On Machine B (Node 2)  
./setup-node2.sh
# Note the destination hash from logs
```

#### Step 3: Exchange Destination Hashes
- Copy Machine A's destination hash to `Config-node2.toml` on Machine B
- Copy Machine B's destination hash to `Config-node1.toml` on Machine A

#### Step 4: Start VPN on Both Machines
```bash
# Machine A
./start-holochain-mesh.sh 1 <MACHINE_B_IP>

# Machine B
./start-holochain-mesh.sh 2 <MACHINE_A_IP>
```

#### Step 5: Verify Connectivity
```bash
# On Machine A
ping 10.0.0.2

# On Machine B
ping 10.0.0.1
```

### Q2: How to connect Volla Messages instances?

**Answer:** Once the VPN is established, Volla Messages will automatically use the mesh network:

1. **Start Volla Messages on both machines**
2. **The apps will discover each other** through the VPN tunnel
3. **Messages will be routed** over the Reticulum mesh network

### Q3: Network Setup Options

#### Option A: Empty Router (Meshnet Only)
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
1. Connect both machines to a router with NO internet access
2. Run RNS-VPN-RS on both machines
3. Start Volla Messages
4. Communication happens entirely over the mesh network

#### Option B: Normal WiFi with Internet
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
1. Connect both machines to normal WiFi with internet
2. Run RNS-VPN-RS on both machines
3. Start Volla Messages
4. Apps will prefer the mesh network for communication

### Q4: How to verify data goes over Reticulum VPN?

**Answer:** Use these verification methods:

#### Method 1: Network Monitoring
```bash
# Monitor Reticulum traffic
sudo tcpdump -i any -n port 4242 or port 4243

# Monitor VPN traffic
sudo tcpdump -i rip0 -n

# Check routing table
ip route show | grep rip0
```

#### Method 2: Traffic Analysis
```bash
# Check if traffic goes through TUN interface
sudo netstat -i
# Look for rip0 interface traffic

# Monitor Reticulum logs
RUST_LOG=debug ./start-holochain-mesh.sh 1 <MACHINE_B_IP>
```

#### Method 3: Disconnect Internet Test
1. Start VPN on both machines
2. **Disconnect internet** from both machines
3. **Volla Messages should still work** between machines
4. This proves communication is over mesh network

### Q5: Internet Sharing via VPN Tunnel

**Answer:** Yes, Machine B can use internet from Machine A via the VPN tunnel.

#### Setup for Internet Sharing:

##### On Machine A (Internet Gateway):
```bash
# Enable IP forwarding
echo 1 | sudo tee /proc/sys/net/ipv4/ip_forward

# Configure NAT for VPN subnet
sudo iptables -t nat -A POSTROUTING -s 10.0.0.0/24 -o <INTERNET_INTERFACE> -j MASQUERADE
sudo iptables -A FORWARD -i rip0 -o <INTERNET_INTERFACE> -j ACCEPT
sudo iptables -A FORWARD -i <INTERNET_INTERFACE> -o rip0 -j ACCEPT
```

##### On Machine B (Client):
```bash
# Set Machine A as gateway
sudo ip route add default via 10.0.0.1 dev rip0

# Test internet access
ping 8.8.8.8
curl https://google.com
```

## Practical Test Scenarios

### Scenario 1: Meshnet Only (No Internet)
```bash
# Setup
1. Connect both machines to router with NO internet
2. Start RNS-VPN-RS on both machines
3. Start Volla Messages on both machines
4. Send messages between machines
5. Verify messages work without internet
```

### Scenario 2: Internet Sharing
```bash
# Setup
1. Machine A has internet, Machine B doesn't
2. Start RNS-VPN-RS on both machines
3. Configure internet sharing on Machine A
4. Test internet access from Machine B
5. Start Volla Messages and verify communication
```

### Scenario 3: Hybrid Communication
```bash
# Setup
1. Both machines have internet
2. Start RNS-VPN-RS on both machines
3. Start Volla Messages on both machines
4. Disconnect internet from one machine
5. Verify messages still work over mesh network
```

## Verification Commands

### Check VPN Status:
```bash
# Check TUN interface
ip addr show rip0

# Check routing
ip route show | grep rip0

# Test connectivity
ping 10.0.0.2  # From Machine A
ping 10.0.0.1  # From Machine B
```

### Monitor Reticulum Traffic:
```bash
# Monitor Reticulum UDP traffic
sudo tcpdump -i any -n port 4242 or port 4243

# Monitor VPN traffic
sudo tcpdump -i rip0 -n
```

### Check Application Traffic:
```bash
# Monitor Volla Messages traffic
sudo tcpdump -i rip0 -n port 443  # HTTPS traffic
sudo tcpdump -i rip0 -n port 80   # HTTP traffic
```

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
