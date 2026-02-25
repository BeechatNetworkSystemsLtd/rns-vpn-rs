# `rns-vpn`

> P2P VPN over Reticulum mesh network

Library and client application for VPN client over Reticulum mesh network.

## Building

Building `Reticulum-rs` requires `protoc` binary for compiling `.proto` files.

Client application is a workspace member, build or run with `-p rns-vpn-client` flag.

## Client configuration

`Config.toml`

`network` -- the network segment to use in CIDR format (e.g. `10.20.0.0/16`)

`peers` -- an initial list of `<destination-hash>` for each peer to communicate with on
the network

## Client application

`rns-vpn-client`

Client application uses a Reticulum UDP or Kaonic interface that is configured with
command-line arguments. See `--help` for all options.

Private keys can be generated with `openssl` tool using the `genkeys.sh` script.

Running with log level INFO will log the destination hash generated for the clients
configured private keys and should be provided to peers to add to their configurations.

Command-line options:

`-p <port>` -- local UDP port for Reticulum interface; required unless using Kaonic
option

`-f <ip>:<port>` -- IP and port for upstream Reticulum node; required unless using
Kaonic option

`-a <grpc-address>` -- Kaonic gRPC address; required unless using UDP options

`[-i <name>]` -- optional: use string to generate private ID; overrides
creation of identity with `RNS_VPN_PRIVKEY_PATH`/`RNS_VPN_SIGNKEY_PATH` variables

Environment variables:

`RNS_VPN_PRIVKEY_PATH` -- path to X25519 private key in PEM format for Reticulum
identity

`RNS_VPN_SIGNKEY_PATH` -- path to ed25519 signing key in PEM format for Reticulum
identity

`RUST_LOG` -- adjust log level: `trace`, `debug`, `info` (default), `warn`, `error`

### Usage

While the client application is running and connected, peers can be reached via their
configured IP addresses and the traffic will be routed over Reticulum. For example a
peer at 10.0.0.1 serving a UDP echo server:
```
# on 10.0.0.1
$ ncat -u -l 10.0.0.1 12345 --sh-exec 'cat'
```
can now be reached via other peers:
```
# on 10.0.0.2
$ ncat -u 10.0.0.1 12345
foo
foo
```
