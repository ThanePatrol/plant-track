let
  moz_overlay = import (builtins.fetchTarball "https://github.com/mozilla/nixpkgs-mozilla/archive/master.tar.gz");
  nixpkgs = import <nixpkgs> { overlays = [ moz_overlay ]; };
in
  with nixpkgs;
  stdenv.mkDerivation {
    name = "moz_overlay_shell";
    buildInputs = [
      (nixpkgs.latest.rustChannels.nightly.rust.override {
          targets = [
            "x86_64-unknown-linux-gnu"
          ];
        })
      cargo-watch
      trunk
      libclang
      openssl
      pkg-config
      libsodium
      zlib
      binutils
      glibc
    ];

  }
