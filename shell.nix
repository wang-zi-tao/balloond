{ pkgs ? import <nixpkgs> { } }:
pkgs.mkShell {
  nativeBuildInputs = with pkgs; [
    pkg-config
    libvirt
    libvirt-glib.dev
  ];

  LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
  RUST_LOG = "debug";
}
