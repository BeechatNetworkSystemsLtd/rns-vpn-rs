# RNS-VPN-RS Repository Guide

## Repository Information

- **Repository**: https://github.com/BeechatNetworkSystemsLtd/rns-vpn-rs
- **Holochain Branch**: `holochain` (contains all setup scripts and documentation)
- **Main Branch**: `master` (basic VPN functionality)

## Branch Structure

### `master` Branch
- Basic RNS-VPN-RS functionality
- Core VPN implementation
- Minimal setup requirements

### `holochain` Branch
- Complete PoC setup with all scripts
- Holochain integration guides
- Comprehensive documentation
- Automated setup scripts
- Test scenarios and verification tools

## Getting Started

### 1. Clone and Checkout Holochain Branch
```bash
git clone https://github.com/BeechatNetworkSystemsLtd/rns-vpn-rs.git
cd rns-vpn-rs
git checkout holochain
```

### 2. Build the Project
```bash
cargo build --release
```

## Documentation Files

### Core Documentation
- **[README.md](README.md)** - Main project documentation
- **[POC_SETUP_GUIDE.md](POC_SETUP_GUIDE.md)** - Complete proof of concept setup guide
- **[TEAM_QUESTIONS_ANSWERS.md](TEAM_QUESTIONS_ANSWERS.md)** - Answers to common setup questions
- **[HOLOCHAIN_SETUP.md](HOLOCHAIN_SETUP.md)** - Holochain integration guide
- **[TEST_RESULTS.md](TEST_RESULTS.md)** - Test results and verification

### Configuration Files
- **Config-node1.toml** - Node 1 VPN configuration
- **Config-node2.toml** - Node 2 VPN configuration
- **holochain-node1.toml** - Holochain config for Node 1
- **holochain-node2.toml** - Holochain config for Node 2

## Setup Scripts

### Node Setup
- **setup-node1.sh** - Automated setup for Node 1
- **setup-node2.sh** - Automated setup for Node 2

### Specialized Setups
- **setup-meshnet-only.sh** - Meshnet-only PoC setup
- **setup-internet-sharing.sh** - Internet sharing setup

### Startup Scripts
- **start-holochain-mesh.sh** - Complete startup script
- **start-meshnet-only.sh** - Meshnet-only startup
- **start-internet-sharing.sh** - Internet sharing startup

### Verification Scripts
- **verify-mesh-traffic.sh** - Traffic verification
- **test-mesh-connectivity.sh** - Connectivity testing

## Quick Start Commands

### Standard Setup (2 machines)
```bash
# Machine A
git clone https://github.com/BeechatNetworkSystemsLtd/rns-vpn-rs.git
cd rns-vpn-rs
git checkout holochain
./setup-node1.sh
./start-holochain-mesh.sh 1 <MACHINE_B_IP>

# Machine B
git clone https://github.com/BeechatNetworkSystemsLtd/rns-vpn-rs.git
cd rns-vpn-rs
git checkout holochain
./setup-node2.sh
./start-holochain-mesh.sh 2 <MACHINE_A_IP>
```

### Meshnet-Only Setup
```bash
# Both machines
git clone https://github.com/BeechatNetworkSystemsLtd/rns-vpn-rs.git
cd rns-vpn-rs
git checkout holochain
sudo ./setup-meshnet-only.sh
sudo ./start-meshnet-only.sh 1 <PEER_IP>  # Machine A
sudo ./start-meshnet-only.sh 2 <PEER_IP>  # Machine B
```

### Internet Sharing Setup
```bash
# Gateway machine (has internet)
git clone https://github.com/BeechatNetworkSystemsLtd/rns-vpn-rs.git
cd rns-vpn-rs
git checkout holochain
sudo ./setup-internet-sharing.sh
sudo ./start-internet-sharing.sh gateway <CLIENT_IP>

# Client machine (no internet)
git clone https://github.com/BeechatNetworkSystemsLtd/rns-vpn-rs.git
cd rns-vpn-rs
git checkout holochain
sudo ./start-internet-sharing.sh client <GATEWAY_IP>
```

## File Structure

```
rns-vpn-rs/
├── src/                          # Source code
├── Config*.toml                  # Configuration files
├── holochain-*.toml             # Holochain configurations
├── setup-*.sh                   # Setup scripts
├── start-*.sh                    # Startup scripts
├── verify-*.sh                  # Verification scripts
├── test-*.sh                    # Testing scripts
├── *.md                         # Documentation
└── Cargo.toml                   # Rust project file
```

## Prerequisites

- Rust toolchain
- OpenSSL
- Root/sudo access (for TUN interface)
- Two machines with network connectivity
- Holochain (for Holochain integration)

## Support

All setup scripts include comprehensive error handling and will guide you through the process. The documentation files provide detailed explanations for each use case.

## Repository Links

- **Main Repository**: https://github.com/BeechatNetworkSystemsLtd/rns-vpn-rs
- **Holochain Branch**: https://github.com/BeechatNetworkSystemsLtd/rns-vpn-rs/tree/holochain
- **Master Branch**: https://github.com/BeechatNetworkSystemsLtd/rns-vpn-rs/tree/master
