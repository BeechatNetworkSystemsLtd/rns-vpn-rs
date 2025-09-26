# Holochain over RNS-VPN-RS Mesh Network Setup

This guide explains how to set up two Holochain nodes communicating over a Reticulum mesh network using RNS-VPN-RS as the transport layer.

## Architecture

```
Node A (10.0.0.1)                    Node B (10.0.0.2)
┌─────────────────┐                  ┌─────────────────┐
│   Holochain     │                  │   Holochain     │
│   Application   │                  │   Application   │
└─────────┬───────┘                  └─────────┬───────┘
          │                                    │
          │ IP Traffic                         │ IP Traffic
          │                                    │
┌─────────▼───────┐                  ┌─────────▼───────┐
│   TUN Interface │                  │   TUN Interface │
│   (10.0.0.1)    │                  │   (10.0.0.2)    │
└─────────┬───────┘                  └─────────┬───────┘
          │                                    │
          │ RNS-VPN-RS                         │ RNS-VPN-RS
          │                                    │
┌─────────▼───────┐                  ┌─────────▼───────┐
│   Reticulum     │◄─────────────────►│   Reticulum     │
│   Mesh Network  │                  │   Mesh Network  │
└─────────────────┘                  └─────────────────┘
```

## Prerequisites

- Two machines with network connectivity
- Rust toolchain installed
- Root/sudo access for TUN interface creation
- Holochain installed on both nodes

## Step 1: Setup RNS-VPN-RS on Both Nodes

### On Node 1 (10.0.0.1):
```bash
# Clone and setup
git clone <your-repo>
cd rns-vpn-rs
./setup-node1.sh

# Start RNS-VPN-RS (replace <NODE2_IP> with actual IP)
RNS_VPN_PRIVKEY_PATH="./node1-privkey.pem" \
RNS_VPN_SIGNKEY_PATH="./node1-signkey.pem" \
RUST_LOG=info \
sudo -E target/release/rns-vpn -p 4242 -f <NODE2_IP>:4243
```

### On Node 2 (10.0.0.2):
```bash
# Clone and setup
git clone <your-repo>
cd rns-vpn-rs
./setup-node2.sh

# Start RNS-VPN-RS (replace <NODE1_IP> with actual IP)
RNS_VPN_PRIVKEY_PATH="./node2-privkey.pem" \
RNS_VPN_SIGNKEY_PATH="./node2-signkey.pem" \
RUST_LOG=info \
sudo -E target/release/rns-vpn -p 4243 -f <NODE1_IP>:4242
```

## Step 2: Exchange Destination Hashes

1. Start both nodes and note the destination hashes from the logs
2. Update the peer configurations:
   - Copy Node 1's destination hash to `Config-node2.toml`
   - Copy Node 2's destination hash to `Config-node1.toml`
3. Restart both RNS-VPN-RS instances

## Step 3: Verify VPN Connectivity

Test IP connectivity between nodes:
```bash
# On Node 1
ping 10.0.0.2

# On Node 2  
ping 10.0.0.1
```

## Step 4: Configure Holochain

### Holochain Network Configuration

Create Holochain conductor configuration files:

#### Node 1 Holochain Config (`holochain-node1.toml`):
```toml
[network]
type = "quic_bootstrap"
bootstrap_server = "https://bootstrap.holo.host"
network_type = "quic"

[network.transport_pools]
default = "quic"

[[network.transport_pools.quic]]
type = "quic"
bind_to = "10.0.0.1:0"  # Bind to VPN interface
```

#### Node 2 Holochain Config (`holochain-node2.toml`):
```toml
[network]
type = "quic_bootstrap"
bootstrap_server = "https://bootstrap.holo.host"
network_type = "quic"

[network.transport_pools]
default = "quic"

[[network.transport_pools.quic]]
type = "quic"
bind_to = "10.0.0.2:0"  # Bind to VPN interface
```

## Step 5: Start Holochain

### On Node 1:
```bash
# Start Holochain conductor
holochain --config holochain-node1.toml
```

### On Node 2:
```bash
# Start Holochain conductor  
holochain --config holochain-node2.toml
```

## Step 6: Test Holochain Communication

Create a simple test application or use existing Holochain apps to verify communication over the mesh network.

## Troubleshooting

### VPN Issues:
- Check TUN interface: `ip addr show`
- Verify routing: `ip route show`
- Test connectivity: `ping 10.0.0.x`

### Holochain Issues:
- Check Holochain logs for network binding errors
- Verify Holochain is binding to VPN interface (10.0.0.x)
- Ensure both nodes can reach each other via VPN

### Reticulum Issues:
- Check Reticulum logs for connection status
- Verify destination hashes are correctly configured
- Ensure UDP ports are not blocked by firewall

## Benefits of This Setup

1. **Decentralized**: No central servers required
2. **Resilient**: Mesh networking provides fault tolerance
3. **Private**: Traffic encrypted over Reticulum
4. **Offline Capable**: Works without internet infrastructure
5. **Scalable**: Can add more nodes to the mesh

## Security Considerations

- Keys are stored in PEM format - ensure proper file permissions
- TUN interface requires root privileges
- Consider firewall rules for additional security
- Monitor network traffic for anomalies
