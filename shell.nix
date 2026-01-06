{ pkgs ? import <nixpkgs> { } }:

let
  rustOverlay = import (builtins.fetchTarball https://github.com/oxalica/rust-overlay/archive/master.tar.gz);
  pkgsWithOverlay = import <nixpkgs> { overlays = [ rustOverlay ]; };
in 
pkgsWithOverlay.mkShell {
  buildInputs = with pkgs; [
    pkgsWithOverlay.rust-bin.stable.latest.default
    gcc
    cargo
    rustup
    rustc
    rust-analyzer
    pkg-config

    vulkan-headers
    vulkan-loader
    vulkan-validation-layers
    vulkan-tools
  ];

  shellHook = ''
    LD_LIBRARY_PATH="''${LD_LIBRARY_PATH:+$LD_LIBRARY_PATH:}${
      with pkgs;
        lib.makeLibraryPath [
          vulkan-loader
          libGL
          libxkbcommon
          wayland
        ]
    }"
    export LD_LIBRARY_PATH
    export RUSTUP_TOOLCHAIN=stable
  '';

  RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
  PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
  WGPU_BACKEND = "Vulkan";
}

