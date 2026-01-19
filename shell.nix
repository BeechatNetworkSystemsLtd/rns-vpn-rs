with import <nixpkgs> {};
let
  unstable = import
    (fetchTarball "https://github.com/NixOS/nixpkgs/archive/nixos-unstable.tar.gz") {};
in
mkShell {
  buildInputs = [
    gdb
    openssl
    protobuf
    rustup
    unstable.nnd
  ];
}
